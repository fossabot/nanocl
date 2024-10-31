use ntex::web;

use nanocl_error::http::HttpResult;
use nanocl_stubs::generic::GenericNspQuery;

use crate::{
  models::{SystemState, VmDb},
  objects::generic::*,
  utils,
};

/// Get detailed information about a virtual machine
#[cfg_attr(feature = "dev", utoipa::path(
  get,
  tag = "Vms",
  path = "/vms/{name}/inspect",
  params(
    ("name" = String, Path, description = "The name of the virtual machine"),
    ("namespace" = Option<String>, Query, description = "Namespace where the virtual machine belongs default to 'global'"),
  ),
  responses(
    (status = 200, description = "Detailed information about a virtual machine", body = nanocl_stubs::vm::VmInspect),
    (status = 404, description = "The virtual machine does not exist", body = crate::services::openapi::ApiError),
  ),
))]
#[web::get("/vms/{name}/inspect")]
pub async fn inspect_vm(
  state: web::types::State<SystemState>,
  path: web::types::Path<(String, String)>,
  qs: web::types::Query<GenericNspQuery>,
) -> HttpResult<web::HttpResponse> {
  let name = path.1.to_owned();
  let namespace = utils::key::resolve_nsp(&qs.namespace);
  let key = utils::key::gen_key(&namespace, &name);
  let vm = VmDb::inspect_obj_by_pk(&key, &state).await?;
  Ok(web::HttpResponse::Ok().json(&vm))
}
