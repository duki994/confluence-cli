//! Top-level command dispatch.
//!
//! Each top-level noun lives in its own module. M0: every handler bails with
//! "not yet implemented". The shape of `Args` and subcommand enums is the
//! real shape — only the bodies will change.

use anyhow::Result;
use clap::Subcommand;

use crate::context::Context;

pub mod api;
pub mod attachment;
pub mod auth;
pub mod comment;
pub mod page;
pub mod search;
pub mod space;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Authenticate with a Confluence host and manage stored credentials.
    Auth(auth::Args),
    /// Read, create, update, and delete pages.
    Page(page::Args),
    /// Search content using CQL.
    Search(search::Args),
    /// List and inspect spaces.
    Space(space::Args),
    /// Upload, download, and list page attachments.
    Attachment(attachment::Args),
    /// Add and list comments on pages.
    Comment(comment::Args),
    /// Make an authenticated request to an arbitrary Confluence REST endpoint.
    Api(api::Args),
}

pub async fn dispatch(command: Command, ctx: &Context) -> Result<()> {
    match command {
        Command::Auth(args) => auth::run(args, ctx).await,
        Command::Page(args) => page::run(args, ctx).await,
        Command::Search(args) => search::run(args, ctx).await,
        Command::Space(args) => space::run(args, ctx).await,
        Command::Attachment(args) => attachment::run(args, ctx).await,
        Command::Comment(args) => comment::run(args, ctx).await,
        Command::Api(args) => api::run(args, ctx).await,
    }
}
