//! `confluence page` — view, list, create, edit, delete pages.
//!
//! M0 stub. Read paths land in M2; write paths in M3.

use anyhow::{bail, Result};
use clap::{Args as ClapArgs, Subcommand};

use crate::context::Context;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    command: PageCommand,
}

#[derive(Debug, Subcommand)]
enum PageCommand {
    /// Print a page by ID.
    View {
        /// Page ID.
        id: String,
        /// Body representation to fetch (`storage` is the only supported value in v0.1).
        #[arg(long, default_value = "storage")]
        body: String,
    },
    /// List pages, optionally filtered by space.
    List {
        /// Space key to list pages within.
        #[arg(long)]
        space: Option<String>,
        /// Maximum number of results to return.
        #[arg(long, default_value_t = 25)]
        limit: u32,
    },
    /// Create a new page.
    Create {
        /// Destination space key.
        #[arg(long)]
        space: String,
        /// Page title.
        #[arg(long)]
        title: String,
        /// Path to a file containing the page body (storage XHTML).
        #[arg(long)]
        file: std::path::PathBuf,
        /// Parent page ID, if any.
        #[arg(long)]
        parent: Option<String>,
    },
    /// Update an existing page.
    Edit {
        /// Page ID.
        id: String,
        /// Path to a file containing the new body (storage XHTML).
        #[arg(long)]
        file: std::path::PathBuf,
        /// Optional version comment recorded with the update.
        #[arg(long)]
        message: Option<String>,
        /// Mark this update as a minor edit.
        #[arg(long)]
        minor_edit: bool,
        /// Override the version-conflict check and force the write.
        #[arg(long)]
        force: bool,
    },
    /// Delete a page.
    Delete {
        /// Page ID.
        id: String,
        /// Skip the interactive confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
}

pub async fn run(args: Args, _ctx: &Context) -> Result<()> {
    match args.command {
        PageCommand::View { .. } => {
            bail!("`page view` is not yet implemented (M0 skeleton; lands in M2)")
        }
        PageCommand::List { .. } => {
            bail!("`page list` is not yet implemented (M0 skeleton; lands in M2)")
        }
        PageCommand::Create { .. } => {
            bail!("`page create` is not yet implemented (M0 skeleton; lands in M3)")
        }
        PageCommand::Edit { .. } => {
            bail!("`page edit` is not yet implemented (M0 skeleton; lands in M3)")
        }
        PageCommand::Delete { .. } => {
            bail!("`page delete` is not yet implemented (M0 skeleton; lands in M3)")
        }
    }
}
