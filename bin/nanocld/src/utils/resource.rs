use ntex::http::StatusCode;
use jsonschema::{JSONSchema, Draft};

use nanocl_stubs::resource::{Resource, ResourcePartial};
use serde_json::Value;

use crate::repositories;
use nanocl_utils::http_error::HttpError;
use crate::models::{Pool, ResourceKindPartial};

use super::ctrl_client::CtrlClient;

/// Validate a resource from a custom config
pub async fn hook_create_resource(
  resource: &ResourcePartial,
  pool: &Pool,
) -> Result<ResourcePartial, HttpError> {
  let mut resource = resource.clone();
  match resource.kind.as_str() {
    "Kind" => {
      // Todo: validate the resource with a structure
      let resource_kind = ResourceKindPartial {
        name: resource.name.to_owned(),
        version: resource.version.to_owned(),
        schema: resource.config.get("Schema").cloned(),
        url: resource.config.get("Url").map(|item| match item {
          Value::String(value) => value.clone(),
          // Wtf ? so if it's not a string, we just convert it to a string ?
          // Meaning that if it's a number an array or whatever, it will be converted to a string ?
          value => value.to_string(),
        }),
      };

      if resource_kind.schema.is_none() && resource_kind.url.is_none() {
        return Err(HttpError {
          msg: "Neither schema nor url provided".to_string(),
          status: StatusCode::BAD_REQUEST,
        });
      }

      if repositories::resource_kind::find_by_name(&resource.name, pool)
        .await
        .is_err()
      {
        repositories::resource_kind::create(&resource_kind, pool).await?;
      }
      repositories::resource_kind::create_version(&resource_kind, pool).await?;
    }
    _ => {
      let kind = repositories::resource_kind::get_version(
        &resource.kind,
        &resource.version,
        pool,
      )
      .await?;

      if let Some(schema) = kind.schema {
        let schema: JSONSchema = JSONSchema::options()
          .with_draft(Draft::Draft7)
          .compile(&schema)
          .map_err(|err| HttpError {
            status: StatusCode::BAD_REQUEST,
            msg: format!("Invalid schema {}", err),
          })?;
        schema.validate(&resource.config).map_err(|err| {
          let mut msg = String::from("Invalid config ");
          for error in err {
            msg += &format!("{} ", error);
          }
          HttpError {
            status: StatusCode::BAD_REQUEST,
            msg,
          }
        })?;
      }

      if let Some(url) = kind.url {
        let ctrl_client = CtrlClient::new(&kind.resource_kind_name, &url);
        let config = ctrl_client
          .apply_rule(&resource.version, &resource.name, &resource.config)
          .await?;
        resource.config = config;
      }
    }
  }
  Ok(resource)
}

/// Hook when deleting a resource
async fn hook_delete_resource(
  resource: &Resource,
  pool: &Pool,
) -> Result<(), HttpError> {
  let kind = repositories::resource_kind::get_version(
    &resource.kind,
    &resource.version,
    pool,
  )
  .await?;
  if let Some(url) = kind.url {
    let ctrl_client = CtrlClient::new(&kind.resource_kind_name, &url);
    ctrl_client
      .delete_rule(&resource.version, &resource.name)
      .await?;
  }

  Ok(())
}

/// Create a resource
pub async fn create(
  resource: &ResourcePartial,
  pool: &Pool,
) -> Result<Resource, HttpError> {
  if repositories::resource::inspect_by_key(&resource.name, pool)
    .await
    .is_ok()
  {
    return Err(HttpError {
      status: StatusCode::CONFLICT,
      msg: format!("Resource {} already exists", &resource.name),
    });
  }

  let resource = hook_create_resource(resource, pool).await?;
  let res = repositories::resource::create(&resource, pool).await?;
  Ok(res)
}

/// Patch a resource
pub async fn patch(
  resource: &ResourcePartial,
  pool: &Pool,
) -> Result<Resource, HttpError> {
  let resource = hook_create_resource(resource, pool).await?;
  let res = repositories::resource::put(&resource, pool).await?;
  Ok(res)
}

/// Delete a resource
pub async fn delete(resource: &Resource, pool: &Pool) -> Result<(), HttpError> {
  if let Err(err) = hook_delete_resource(resource, pool).await {
    log::warn!("{err}");
  }
  if resource.kind.as_str() == "Kind" {
    repositories::resource_kind::delete_version(&resource.name, pool).await?;
    repositories::resource_kind::delete(&resource.name, pool).await?;
  }
  repositories::resource::delete_by_key(&resource.name, pool).await?;
  repositories::resource_config::delete_by_resource_key(&resource.name, pool)
    .await?;
  Ok(())
}
