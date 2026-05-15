//! `confluence space` — list and inspect spaces.
//!
//! M0 stub. Lands in M2.

use anyhow::{bail, Result};
use clap::{Args as ClapArgs, Subcommand};

use crate::context::Context;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    command: SpaceCommand,
}

#[derive(Debug, Subcommand)]
enum SpaceCommand {
    /// List spaces visible to the active user.
    List {
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
    /// Show details for a single space.
    View {
        /// Space key.
        key: String,
    },
}

pub async fn run(args: Args, _ctx: &Context) -> Result<()> {
    match args.command {
        SpaceCommand::List { .. } => {
            bail!("`space list` is not yet implemented (M0 skeleton; lands in M2)")
        }
        SpaceCommand::View { .. } => {
            bail!("`space view` is not yet implemented (M0 skeleton; lands in M2)")
        }
    }
}
