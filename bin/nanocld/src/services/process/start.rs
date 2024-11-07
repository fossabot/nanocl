use ntex::web;

use bollard_next::container::StartContainerOptions;
use nanocl_error::http::{HttpError, HttpResult};
use nanocl_stubs::generic::GenericNspQuery;

use crate::{
  models::{ProcessDb, SystemState},
  repositories::generic::*,
  utils,
};

/// Start a single process by it's name or id
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  tag = "Processes",
  path = "/processes/{name}/start",
  params(
    ("name" = String, Path, description = "Name of the process", example = "ndaemon.system.c"),
  ),
  responses(
    (status = 202, description = "Process started"),
  ),
))]
#[web::post("/processes/{name}/start")]
pub async fn start_process_by_pk(
  state: web::types::State<SystemState>,
  path: web::types::Path<(String, String)>,
) -> HttpResult<web::HttpResponse> {
  let (_, pk) = path.into_inner();
  let process = ProcessDb::read_by_pk(&pk, &state.inner.pool).await?;
  state
    .inner
    .docker_api
    .start_container(&process.key, None::<StartContainerOptions<String>>)
    .await?;
  Ok(web::HttpResponse::Accepted().finish())
}

/// Start all processes of given kind and name (cargo, job, vm)
/// This emit and event to start all instances of a process and will return before the process is actually started
/// The reason is if we have 1000 instances of a process, we don't want to wait for all of them to start it may result in a timeout
///
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  tag = "Processes",
  path = "/processes/{kind}/{name}/start",
  params(
    ("kind" = String, Path, description = "Kind of the process", example = "cargo"),
    ("name" = String, Path, description = "Name of the process", example = "deploy-example"),
    ("namespace" = Option<String>, Query, description = "Namespace where the process belongs if needed"),
  ),
  responses(
    (status = 202, description = "Process instances starting"),
  ),
))]
#[web::post("/processes/{kind}/{name}/start")]
pub async fn start_processes(
  state: web::types::State<SystemState>,
  path: web::types::Path<(String, String, String)>,
  qs: web::types::Query<GenericNspQuery>,
) -> HttpResult<web::HttpResponse> {
  let (_, kind, name) = path.into_inner();
  let kind = kind.parse().map_err(HttpError::bad_request)?;
  let kind_key = utils::key::gen_kind_key(&kind, &name, &qs.namespace);
  utils::container::generic::emit_starting(&kind_key, &kind, &state).await?;
  Ok(web::HttpResponse::Accepted().finish())
}
