use ntex::web;

use nanocl_error::http::{HttpError, HttpResult};
use nanocl_stubs::generic::GenericNspQuery;

use crate::{models::SystemState, utils};

#[cfg_attr(feature = "dev", utoipa::path(
  post,
  tag = "Processes",
  path = "/processes/{name}/stop",
  params(
    ("name" = String, Path, description = "Name of the process", example = "deploy-example"),
  ),
  responses(
    (status = 202, description = "Process instances stopped"),
  )
))]
#[web::post("/processes/{name}/stop")]
pub async fn stop_process(
  state: web::types::State<SystemState>,
  path: web::types::Path<(String, String)>,
) -> HttpResult<web::HttpResponse> {
  let (_, name) = path.into_inner();
  state.inner.docker_api.stop_container(&name, None).await?;
  Ok(web::HttpResponse::Accepted().finish())
}

/// Stop all processes of given kind and name (cargo, job, vm)
#[cfg_attr(feature = "dev", utoipa::path(
  post,
  tag = "Processes",
  path = "/processes/{kind}/{name}/stop",
  params(
    ("kind" = String, Path, description = "Kind of the process", example = "cargo"),
    ("name" = String, Path, description = "Name of the process", example = "deploy-example"),
    ("namespace" = Option<String>, Query, description = "Namespace where the process belongs if needed"),
  ),
  responses(
    (status = 202, description = "Process instances stopped"),
  ),
))]
#[web::post("/processes/{kind}/{name}/stop")]
pub async fn stop_processes(
  state: web::types::State<SystemState>,
  path: web::types::Path<(String, String, String)>,
  qs: web::types::Query<GenericNspQuery>,
) -> HttpResult<web::HttpResponse> {
  let (_, kind, name) = path.into_inner();
  let kind = kind.parse().map_err(HttpError::bad_request)?;
  let kind_key = utils::key::gen_kind_key(&kind, &name, &qs.namespace);
  utils::container::generic::emit_stopping(&kind_key, &kind, &state).await?;
  Ok(web::HttpResponse::Accepted().finish())
}
