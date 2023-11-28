use std::sync::Arc;

use ntex::web;
use diesel::prelude::*;

use nanocl_error::io::{IoError, IoResult, FromIo};
use nanocl_stubs::generic::GenericDelete;
use nanocl_stubs::vm_spec::{VmSpec, VmSpecPartial};

use crate::utils;
use crate::models::{Pool, VmSpecDb, FromSpec};

/// ## Create
///
/// Create a vm spec item in database for given `VmSpecPartial`
/// and return a `VmSpec` with the generated key
///
/// ## Arguments
///
/// * [vm_key](str) - Vm key
/// * [item](VmSpecPartial) - Vm spec item
/// * [version](str) - Vm spec version
/// * [pool](Pool) - Database connection pool
///
/// ## Return
///
/// [IoResult](IoResult) containing a [VmSpec](VmSpec)
///
pub(crate) async fn create(
  vm_key: &str,
  item: &VmSpecPartial,
  version: &str,
  pool: &Pool,
) -> IoResult<VmSpec> {
  let db_model = VmSpecDb::try_from_spec_partial(vm_key, version, item)?;
  let db_model: VmSpecDb =
    super::generic::insert_with_res(db_model, pool).await?;
  Ok(db_model.into_spec(item))
}

/// ## Find by key
///
/// Find a vm spec item in database for given key
///
/// ## Arguments
///
/// * [key](uuid::Uuid) - Vm spec key
/// * [pool](Pool) - Database connection pool
///
/// ## Return
///
/// [IoResult](IoResult) containing a [VmSpec](VmSpec)
///
pub(crate) async fn find_by_key(
  key: &uuid::Uuid,
  pool: &Pool,
) -> IoResult<VmSpec> {
  use crate::schema::vm_specs;
  let key = *key;
  let dbmodel =
    super::generic::find_by_id::<vm_specs::table, _, VmSpecDb>(key, pool)
      .await?;
  dbmodel.try_to_spec()
}

/// ## Delete by vm key
///
/// Delete all vm spec items in database for given vm key
///
/// ## Arguments
///
/// * [key](str) - Vm key
/// * [pool](Pool) - Database connection pool
///
/// ## Return
///
/// [IoResult](IoResult) containing a [GenericDelete](GenericDelete)
///
pub(crate) async fn delete_by_vm_key(
  key: &str,
  pool: &Pool,
) -> IoResult<GenericDelete> {
  use crate::schema::vm_specs;
  let key = key.to_owned();
  let pool = Arc::clone(pool);
  let res = web::block(move || {
    let mut conn = utils::store::get_pool_conn(&pool)?;
    let res = diesel::delete(vm_specs::dsl::vm_specs)
      .filter(vm_specs::dsl::vm_key.eq(key))
      .execute(&mut conn)
      .map_err(|err| err.map_err_context(|| "VmSpec"))?;
    Ok::<_, IoError>(res)
  })
  .await?;
  Ok(GenericDelete { count: res })
}

/// ## List by vm key
///
/// List all vm spec items in database for given vm key
///
/// ## Arguments
///
/// * [key](str) - Vm key
/// * [pool](Pool) - Database connection pool
///
/// ## Return
///
/// [IoResult](IoResult) containing a [Vec](Vec) of [VmSpec](VmSpec)
///
pub(crate) async fn list_by_vm_key(
  key: &str,
  pool: &Pool,
) -> IoResult<Vec<VmSpec>> {
  use crate::schema::vm_specs;
  let key = key.to_owned();
  let pool = Arc::clone(pool);
  let dbmodels = web::block(move || {
    let mut conn = utils::store::get_pool_conn(&pool)?;
    let specs = vm_specs::dsl::vm_specs
      .filter(vm_specs::dsl::vm_key.eq(key))
      .get_results::<VmSpecDb>(&mut conn)
      .map_err(|err| err.map_err_context(|| "VmSpec"))?;
    Ok::<_, IoError>(specs)
  })
  .await?;
  let specs = dbmodels
    .into_iter()
    .map(|dbmodel: VmSpecDb| dbmodel.try_to_spec())
    .collect::<Result<Vec<VmSpec>, IoError>>()?;
  Ok(specs)
}
