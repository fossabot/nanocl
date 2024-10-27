use nanocl_error::io::IoResult;
use nanocld_client::stubs::process::{Process, ProcessLogQuery};

use crate::{
  config::CliConfig,
  models::{GenericListOpts, LogsOpts, ProcessArg, ProcessFilter, ProcessRow},
  utils,
};

use super::{GenericCommand, GenericCommandLs};

impl GenericCommand for ProcessArg {
  fn object_name() -> &'static str {
    "processes"
  }
}

impl GenericCommandLs for ProcessArg {
  type Item = ProcessRow;
  type Args = ProcessArg;
  type ApiItem = Process;

  fn get_key(item: &Self::Item) -> String {
    item.key.clone()
  }
}

/// Get logs for a process by name or id
pub async fn logs_process(
  cli_conf: &CliConfig,
  opts: &LogsOpts,
) -> IoResult<()> {
  let query: ProcessLogQuery = opts.clone().into();
  let stream = cli_conf
    .client
    .logs_process(&opts.name, Some(&query))
    .await?;
  utils::print::logs_process_stream(stream).await?;
  Ok(())
}

pub async fn exec_process(
  cli_conf: &CliConfig,
  opts: &GenericListOpts<ProcessFilter>,
) -> IoResult<()> {
  let args = &ProcessArg;
  ProcessArg::exec_ls(&cli_conf.client, args, opts).await
}
