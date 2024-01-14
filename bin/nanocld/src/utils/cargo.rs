use futures::StreamExt;
use futures_util::stream::FuturesUnordered;

use nanocl_error::{
  io::{IoResult, IoError},
  http::{HttpResult, HttpError},
};

use bollard_next::{
  service::{HostConfig, RestartPolicy, RestartPolicyNameEnum},
  container::{
    StartContainerOptions, WaitContainerOptions, RemoveContainerOptions,
  },
};
use nanocl_stubs::{
  cargo::Cargo,
  process::{Process, ProcessKind},
  system::{EventPartial, EventActorKind, EventActor, EventKind},
  generic::{GenericFilter, GenericClause},
};

use crate::{
  vars, utils,
  repositories::generic::*,
  models::{SystemState, CargoDb, SecretDb},
  objects::generic::ObjProcess,
};

/// Container to execute before the cargo instances
async fn execute_before(cargo: &Cargo, state: &SystemState) -> HttpResult<()> {
  match cargo.spec.init_container.clone() {
    Some(mut before) => {
      let image = before
        .image
        .clone()
        .unwrap_or(cargo.spec.container.image.clone().unwrap());
      before.image = Some(image);
      before.host_config = Some(HostConfig {
        network_mode: Some(cargo.namespace_name.clone()),
        ..before.host_config.unwrap_or_default()
      });
      let mut labels = before.labels.to_owned().unwrap_or_default();
      labels.insert("io.nanocl.c".to_owned(), cargo.spec.cargo_key.to_owned());
      labels.insert("io.nanocl.n".to_owned(), cargo.namespace_name.to_owned());
      labels.insert("io.nanocl.init-c".to_owned(), "true".to_owned());
      labels.insert(
        "com.docker.compose.project".into(),
        format!("nanocl_{}", cargo.namespace_name),
      );
      before.labels = Some(labels);
      let short_id = utils::key::generate_short_id(6);
      let name = format!(
        "init-{}-{}.{}.c",
        cargo.spec.name, short_id, cargo.namespace_name
      );
      utils::container::create_process(
        &ProcessKind::Cargo,
        &name,
        &cargo.spec.cargo_key,
        before,
        state,
      )
      .await?;
      state
        .docker_api
        .start_container(&name, None::<StartContainerOptions<String>>)
        .await?;
      let options = Some(WaitContainerOptions {
        condition: "not-running",
      });
      let mut stream = state.docker_api.wait_container(&name, options);
      while let Some(wait_status) = stream.next().await {
        log::trace!("init_container: wait {wait_status:?}");
        match wait_status {
          Ok(wait_status) => {
            log::debug!("Wait status: {wait_status:?}");
            if wait_status.status_code != 0 {
              let error = match wait_status.error {
                Some(error) => error.message.unwrap_or("Unknown error".into()),
                None => "Unknown error".into(),
              };
              return Err(HttpError::internal_server_error(format!(
                "Error while waiting for before container: {error}"
              )));
            }
          }
          Err(err) => {
            return Err(HttpError::internal_server_error(format!(
              "Error while waiting for before container: {err}"
            )));
          }
        }
      }
      Ok(())
    }
    None => Ok(()),
  }
}

/// Create instances (containers) based on the cargo spec
/// The number of containers created is based on the number of instances
/// defined in the cargo spec
/// If the number of instances is greater than 1, the containers will be named
/// with the cargo key and a number
/// Example: cargo-key-1, cargo-key-2, cargo-key-3
/// If the number of instances is equal to 1, the container will be named with
/// the cargo key.
pub async fn create_instances(
  cargo: &Cargo,
  number: usize,
  state: &SystemState,
) -> HttpResult<Vec<Process>> {
  download_image(cargo, state).await?;
  execute_before(cargo, state).await?;
  let mut secret_envs: Vec<String> = Vec::new();
  if let Some(secrets) = &cargo.spec.secrets {
    let filter = GenericFilter::new()
      .r#where("key", GenericClause::In(secrets.clone()))
      .r#where("kind", GenericClause::Eq("nanocl.io/env".to_owned()));
    let secrets = SecretDb::transform_read_by(&filter, &state.pool)
      .await?
      .into_iter()
      .map(|secret| {
        let envs = serde_json::from_value::<Vec<String>>(secret.data)?;
        Ok::<_, IoError>(envs)
      })
      .collect::<IoResult<Vec<Vec<String>>>>()?;
    // Flatten the secrets to have envs in a single vector
    secret_envs = secrets.into_iter().flatten().collect();
  }
  (0..number)
    .collect::<Vec<usize>>()
    .into_iter()
    .map(move |current| {
      let secret_envs = secret_envs.clone();
      async move {
        let short_id = utils::key::generate_short_id(6);
        let name = format!("{}-{}.{}.c", cargo.spec.name, short_id, cargo.namespace_name);
        let spec = cargo.spec.clone();
        let container = spec.container;
        let host_config = container.host_config.unwrap_or_default();
        // Add cargo label to the container to track it
        let mut labels = container.labels.to_owned().unwrap_or_default();
        labels.insert("io.nanocl.c".to_owned(), cargo.spec.cargo_key.to_owned());
        labels
          .insert("io.nanocl.n".to_owned(), cargo.namespace_name.to_owned());
        labels.insert(
          "com.docker.compose.project".into(),
          format!("nanocl_{}", cargo.namespace_name),
        );
        let auto_remove =
          host_config
          .auto_remove
          .unwrap_or(false);
        if auto_remove {
          return Err(HttpError::bad_request("Using autoremove for a cargo is not allowed, consider using a job instead"));
        }
        let restart_policy =
          Some(
              host_config
              .restart_policy
              .unwrap_or(RestartPolicy {
                name: Some(RestartPolicyNameEnum::ALWAYS),
                maximum_retry_count: None,
              }),
          );
        let mut env = container.env.unwrap_or_default();
        // merge cargo env with secret env
        env.extend(secret_envs);
        let hostname = match cargo.spec.container.hostname {
          Some(ref hostname) => {
            format!("{hostname}-{short_id}")
          }
          None => name.to_owned(),
        };
        env.push(format!("NANOCL_NODE={}", state.config.hostname));
        env.push(format!("NANOCL_NODE_ADDR={}", state.config.gateway));
        env.push(format!("NANOCL_CARGO_KEY={}", cargo.spec.cargo_key.to_owned()));
        env.push(format!("NANOCL_CARGO_NAMESPACE={}", cargo.namespace_name));
        env.push(format!("NANOCL_CARGO_INSTANCE={}", current));
        // Merge the cargo spec with the container spec
        // And set his network mode to the cargo namespace
        let new_process = bollard_next::container::Config {
          attach_stderr: Some(true),
          attach_stdout: Some(true),
          tty: Some(true),
          hostname: Some(hostname),
          labels: Some(labels),
          env: Some(env),
          host_config: Some(HostConfig {
            restart_policy,
            network_mode: Some(
                host_config
                .network_mode
                .unwrap_or(cargo.namespace_name.clone()),
            ),
            ..host_config
          }),
          ..container
        };
        utils::container::create_process(
          &ProcessKind::Cargo,
          &name,
          &cargo.spec.cargo_key,
          new_process,
          state,
        ).await
      }
    })
    .collect::<FuturesUnordered<_>>()
    .collect::<Vec<HttpResult<Process>>>()
    .await
    .into_iter()
    .collect::<HttpResult<Vec<Process>>>()
}

/// The instances (containers) are deleted but the cargo is not.
/// The cargo is not deleted because it can be used to restore the containers.
pub async fn delete_instances(
  instances: &[String],
  state: &SystemState,
) -> HttpResult<()> {
  instances
    .iter()
    .map(|id| async {
      CargoDb::del_process_by_pk(
        id,
        Some(RemoveContainerOptions {
          force: true,
          ..Default::default()
        }),
        state,
      )
      .await
    })
    .collect::<FuturesUnordered<_>>()
    .collect::<Vec<HttpResult<()>>>()
    .await
    .into_iter()
    .collect::<HttpResult<()>>()
}

pub async fn download_image(
  cargo: &Cargo,
  state: &SystemState,
) -> HttpResult<()> {
  let image_name = &cargo.spec.container.image.clone().unwrap_or_default();
  if state.docker_api.inspect_image(image_name).await.is_ok() {
    return Ok(());
  }
  let (name, tag) = utils::container_image::parse_name(image_name)?;
  let mut stream = state.docker_api.create_image(
    Some(bollard_next::image::CreateImageOptions {
      from_image: name.clone(),
      tag: tag.clone(),
      ..Default::default()
    }),
    None,
    None,
  );
  while let Some(chunk) = stream.next().await {
    let chunk = match chunk {
      Err(err) => {
        let event = EventPartial {
          reporting_controller: vars::CONTROLLER_NAME.to_owned(),
          reporting_node: state.config.hostname.clone(),
          action: "download_image".to_owned(),
          reason: "state_sync".to_owned(),
          kind: EventKind::Error,
          actor: Some(EventActor {
            key: cargo.spec.container.image.clone(),
            kind: EventActorKind::ContainerImage,
            attributes: None,
          }),
          related: Some(cargo.clone().into()),
          note: Some(format!(
            "Error while downloading image {image_name} {err}"
          )),
          metadata: None,
        };
        state.spawn_emit_event(event);
        return Err(err.into());
      }
      Ok(chunk) => chunk,
    };
    let event = EventPartial {
      reporting_controller: vars::CONTROLLER_NAME.to_owned(),
      reporting_node: state.config.hostname.clone(),
      action: "download_image".to_owned(),
      reason: "state_sync".to_owned(),
      kind: EventKind::Normal,
      actor: Some(EventActor {
        key: cargo.spec.container.image.clone(),
        kind: EventActorKind::ContainerImage,
        attributes: None,
      }),
      related: Some(cargo.clone().into()),
      note: Some(format!("Downloading image {name}:{tag}")),
      metadata: Some(serde_json::json!({
        "state": chunk,
      })),
    };
    state.spawn_emit_event(event);
  }
  Ok(())
}
