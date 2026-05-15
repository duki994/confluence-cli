//! `confluence auth` — manage stored credentials and the active host.
//!
//! M0 stub. Real implementation lands in M1, see
//! `.claude/agents/confluence-auth-rust-developer.md`.

use anyhow::{bail, Result};
use clap::{Args as ClapArgs, Subcommand};

use crate::context::Context;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    command: AuthCommand,
}

#[derive(Debug, Subcommand)]
enum AuthCommand {
    /// Log in to a Confluence host with an API token.
    Login,
    /// Remove a stored credential.
    Logout,
    /// Show the active host and any other configured hosts.
    Status,
    /// Switch the active host.
    Switch {
        /// Host to make active (e.g. `your-org.atlassian.net`).
        host: String,
    },
}

pub async fn run(args: Args, _ctx: &Context) -> Result<()> {
    match args.command {
        AuthCommand::Login => {
            bail!("`auth login` is not yet implemented (M0 skeleton; lands in M1)")
        }
        AuthCommand::Logout => {
            bail!("`auth logout` is not yet implemented (M0 skeleton; lands in M1)")
        }
        AuthCommand::Status => {
            bail!("`auth status` is not yet implemented (M0 skeleton; lands in M1)")
        }
        AuthCommand::Switch { .. } => {
            bail!("`auth switch` is not yet implemented (M0 skeleton; lands in M1)")
        }
    }
}
