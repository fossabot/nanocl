mod cli;
mod models;
mod version;
mod client;
mod utils;
mod config;

use clap::Parser;
use cli::errors::CliError;

use models::*;

fn process_error(args: &Cli, err: CliError) {
  match err {
    CliError::Client(err) => match err {
      client::error::NanocldError::SendRequest(err) => match err {
        ntex::http::client::error::SendRequestError::Connect(_) => {
          eprintln!(
            "Cannot connect to the nanocl daemon at {host}. Is the nanocl daemon running?",
            host = args.host
          )
        }
        _ => eprintln!("{}", err),
      },
      client::error::NanocldError::Api(err) => {
        eprintln!("Daemon [{}]: {}", err.status, err.msg);
      }
      _ => eprintln!("{}", err),
    },
    _ => eprintln!("{}", err),
  }
  std::process::exit(1);
}

async fn execute_args(args: &Cli) -> Result<(), CliError> {
  let client = client::Nanocld::connect_with_unix_default().await;
  match &args.command {
    Commands::Setup(args) => cli::exec_setup(args).await,
    Commands::Namespace(args) => cli::exec_namespace(&client, args).await,
    Commands::Cargo(args) => cli::exec_cargo(&client, args).await,
    Commands::Version => cli::exec_version(&client).await,
  }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
  let args = Cli::parse();
  if let Err(err) = execute_args(&args).await {
    process_error(&args, err);
  }
  Ok(())
}
