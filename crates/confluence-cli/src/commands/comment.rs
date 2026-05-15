//! `confluence comment` — page comments.
//!
//! M0 stub. Lands in M4.

use anyhow::{bail, Result};
use clap::{Args as ClapArgs, Subcommand};
use std::path::PathBuf;

use crate::context::Context;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    command: CommentCommand,
}

#[derive(Debug, Subcommand)]
enum CommentCommand {
    /// Add a comment to a page.
    Add {
        page_id: String,
        /// File containing the comment body (storage XHTML).
        #[arg(long)]
        file: PathBuf,
    },
    /// List comments on a page.
    List { page_id: String },
}

pub async fn run(args: Args, _ctx: &Context) -> Result<()> {
    match args.command {
        CommentCommand::Add { .. } => {
            bail!("`comment add` is not yet implemented (M0 skeleton; lands in M4)")
        }
        CommentCommand::List { .. } => {
            bail!("`comment list` is not yet implemented (M0 skeleton; lands in M4)")
        }
    }
}
