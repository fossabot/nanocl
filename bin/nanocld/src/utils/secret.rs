use nanocl_error::io::{IoError, IoResult};
use nanocl_stubs::generic::{GenericClause, GenericFilter};

use crate::{
  models::{SecretDb, SystemState},
  repositories::generic::*,
};

/// Transform and optional vector of secrets to a vector of envs from the database
///
pub async fn load_env_secrets(
  secrets: &Option<Vec<String>>,
  state: &SystemState,
) -> IoResult<Vec<String>> {
  let mut env_secrets: Vec<String> = Vec::new();
  if let Some(secrets) = &secrets {
    let filter = GenericFilter::new()
      .r#where("key", GenericClause::In(secrets.clone()))
      .r#where("kind", GenericClause::Eq("nanocl.io/env".to_owned()));
    let secrets = SecretDb::transform_read_by(&filter, &state.inner.pool)
      .await?
      .into_iter()
      .map(|secret| {
        let envs = serde_json::from_value::<Vec<String>>(secret.data)?;
        Ok::<_, IoError>(envs)
      })
      .collect::<IoResult<Vec<Vec<String>>>>()?;
    // Flatten the secrets to have envs in a single vector
    env_secrets = secrets.into_iter().flatten().collect();
  }
  Ok(env_secrets)
}
