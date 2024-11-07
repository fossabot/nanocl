use ntex::web;

use bollard_next::container::Config;
use nanocl_error::http::{HttpError, HttpResult};
use nanocl_stubs::{generic::GenericNspQuery, process::ProcessKind};

use crate::{
  models::{CargoDb, JobDb, SystemState, VmDb},
  repositories::generic::RepositoryReadByTransform,
  utils,
};

#[cfg_attr(feature = "dev", utoipa::path(
  post,
  tag = "Processes",
  path = "/processes",
  responses(
    (status = 201, description = "Process created"),
    (status = 400, description = "Bad request", body = crate::services::openapi::ApiError),
  ),
))]
#[web::post("/processes")]
pub async fn create_process(
  state: web::types::State<SystemState>,
  body: web::types::Json<Config>,
  path: web::types::Path<(String, String, String)>,
  qs: web::types::Query<GenericNspQuery>,
) -> HttpResult<web::HttpResponse> {
  // Ok(web::HttpResponse::Created().json(&process))
}
