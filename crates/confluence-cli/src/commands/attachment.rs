//! `confluence attachment` — upload, download, list attachments.
//!
//! M0 stub. Lands in M4.

use anyhow::{bail, Result};
use clap::{Args as ClapArgs, Subcommand};
use std::path::PathBuf;

use crate::context::Context;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    command: AttachmentCommand,
}

#[derive(Debug, Subcommand)]
enum AttachmentCommand {
    /// Upload a file as an attachment to a page.
    Upload { page_id: String, file: PathBuf },
    /// List attachments on a page.
    List { page_id: String },
    /// Download an attachment by ID.
    Download {
        attachment_id: String,
        /// Output path (defaults to the attachment's filename).
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

pub async fn run(args: Args, _ctx: &Context) -> Result<()> {
    match args.command {
        AttachmentCommand::Upload { .. } => {
            bail!("`attachment upload` is not yet implemented (M0 skeleton; lands in M4)")
        }
        AttachmentCommand::List { .. } => {
            bail!("`attachment list` is not yet implemented (M0 skeleton; lands in M4)")
        }
        AttachmentCommand::Download { .. } => {
            bail!("`attachment download` is not yet implemented (M0 skeleton; lands in M4)")
        }
    }
}
