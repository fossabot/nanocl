use ntex::web;

use nanocl_error::{http::HttpResult, io::IoResult};
use nanocl_stubs::generic::GenericNspQuery;

use crate::{
  models::{SpecDb, SystemState},
  utils,
};

/// List virtual machine histories
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  tag = "Vms",
  path = "/vms/{name}/histories",
  params(
    ("name" = String, Path, description = "The name of the virtual machine"),
    ("namespace" = Option<String>, Query, description = "Namespace where the virtual machine belongs default to 'global'"),
  ),
  responses(
    (status = 200, description = "The virtual machine histories have been listed", body = [nanocl_stubs::vm_spec::VmSpec]),
    (status = 404, description = "The virtual machine does not exist", body = crate::services::openapi::ApiError),
  ),
))]
#[web::get("/vms/{name}/histories")]
pub async fn list_vm_history(
  state: web::types::State<SystemState>,
  path: web::types::Path<(String, String)>,
  qs: web::types::Query<GenericNspQuery>,
) -> HttpResult<web::HttpResponse> {
  let namespace = utils::key::resolve_nsp(&qs.namespace);
  let key = utils::key::gen_key(&namespace, &path.1);
  let histories = SpecDb::read_by_kind_key(&key, &state.inner.pool)
    .await?
    .into_iter()
    .map(|i| i.try_to_vm_spec())
    .collect::<IoResult<Vec<_>>>()?;
  Ok(web::HttpResponse::Ok().json(&histories))
}
