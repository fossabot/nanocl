use ntex::web;

use nanocl_error::http::{HttpError, HttpResult};
use nanocl_stubs::{generic::GenericNspQuery, process::ProcessKind};

use crate::{
  models::{CargoDb, JobDb, SystemState, VmDb, VmImageDb},
  repositories::generic::*,
  utils,
};

#[cfg_attr(feature = "dev", utoipa::path(
  post,
  tag = "Processes",
  path = "/processes/{kind}/{name}",
  params(
    ("kind" = String, Path, description = "Kind of the process", example = "cargo"),
    ("name" = String, Path, description = "Name of the process", example = "deploy-example"),
    ("namespace" = Option<String>, Query, description = "Namespace where the process belongs if needed"),
  ),
  responses(
    (status = 201, description = "Process created"),
    (status = 400, description = "Bad request", body = crate::services::openapi::ApiError),
  ),
))]
#[web::post("/processes")]
pub async fn create_process(
  state: web::types::State<SystemState>,
  path: web::types::Path<(String, String, String)>,
  qs: web::types::Query<GenericNspQuery>,
) -> HttpResult<web::HttpResponse> {
  let (_, kind, name) = path.into_inner();
  let kind = kind.parse().map_err(HttpError::bad_request)?;
  let kind_key = utils::key::gen_kind_key(&kind, &name, &qs.namespace);
  let processes = match kind {
    ProcessKind::Cargo => {
      let cargo =
        CargoDb::transform_read_by_pk(&kind_key, &state.inner.pool).await?;
      utils::container::cargo::create(&cargo, 1, &state).await?
    }
    ProcessKind::Job => {
      let job =
        JobDb::transform_read_by_pk(&kind_key, &state.inner.pool).await?;
      utils::container::job::create(&job, &state).await?
    }
    ProcessKind::Vm => {
      let vm = VmDb::transform_read_by_pk(&kind_key, &state.inner.pool).await?;
      let vm_image =
        VmImageDb::read_by_pk(&vm.spec.disk.image, &state.inner.pool).await?;
      let process =
        utils::container::vm::create(&vm, &vm_image, true, &state).await?;
      vec![process]
    }
  };
  Ok(web::HttpResponse::Created().json(&processes))
}
