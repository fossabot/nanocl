use std::collections::HashMap;

use chrono::Utc;
use tabled::Tabled;
use chrono::DateTime;
use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};

use super::utils::serde::*;
use super::utils::tabled::*;

#[derive(Debug, Parser)]
pub struct CargoImageRemoveOpts {
  /// id or name of image to delete
  pub(crate) name: String,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct CargoImagePartial {
  pub(crate) name: String,
}

#[derive(Debug, Parser)]
pub struct CargoImageDeployOpts {
  pub(crate) name: String,
}

#[derive(Debug, Parser)]
pub struct CargoImageInspectOpts {
  pub(crate) name: String,
}

#[derive(Debug, Subcommand)]
pub enum CargoImageCommands {
  /// List cargo images
  #[clap(alias("ls"))]
  List,
  /// Create a new cargo image
  Create(CargoImagePartial),
  /// Remove an existing cargo image
  #[clap(alias("rm"))]
  Remove(CargoImageRemoveOpts),
  // #[clap(alias("dp"))]
  // Deploy(CargoImageDeployOpts),
  /// Inspect a cargo image
  Inspect(CargoImageInspectOpts),
}

/// Manage container images
#[derive(Debug, Parser)]
pub struct CargoImageArgs {
  #[clap(subcommand)]
  pub(crate) commands: CargoImageCommands,
}
