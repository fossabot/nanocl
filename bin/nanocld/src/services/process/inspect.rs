use ntex::web;

use nanocl_error::http::HttpResult;
use nanocl_stubs::{
  generic::{GenericClause, GenericFilter},
  process::Process,
};

use crate::{
  models::{ProcessDb, SystemState},
  repositories::generic::*,
};

/// Get detailed information about a process by it's name
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  tag = "Processes",
  path = "/processes/{name}/inspect",
  params(
    ("name" = String, Path, description = "Name of the process"),
  ),
  responses(
    (status = 200, description = "Cargo details", body = Process),
  ),
))]
#[web::get("/processes/{name}/inspect")]
pub async fn inspect_process(
  state: web::types::State<SystemState>,
  path: web::types::Path<(String, String)>,
) -> HttpResult<web::HttpResponse> {
  let filter =
    GenericFilter::new().r#where("name", GenericClause::Eq(path.1.clone()));
  let process: Process = ProcessDb::read_one_by(&filter, &state.inner.pool)
    .await?
    .try_into()?;
  Ok(web::HttpResponse::Ok().json(&process))
}
