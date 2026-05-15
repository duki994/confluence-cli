//! `confluence api` — generic authenticated request to any Confluence REST endpoint.
//!
//! Ships in M1, after the request infrastructure is real.

use anyhow::{bail, Result};
use clap::Args as ClapArgs;

use crate::context::Context;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// HTTP method (defaults to GET; use POST/PUT/PATCH/DELETE for writes).
    #[arg(long, default_value = "GET")]
    method: String,
    /// Endpoint path, e.g. `/wiki/api/v2/pages/123` or `/wiki/rest/api/search`.
    path: String,
    /// Request body (raw string). Mutually exclusive with `--input`.
    #[arg(long, conflicts_with = "input")]
    body: Option<String>,
    /// Path to a file whose contents will be sent as the request body.
    #[arg(long)]
    input: Option<std::path::PathBuf>,
}

pub async fn run(_args: Args, _ctx: &Context) -> Result<()> {
    bail!("`api` is not yet implemented (M0 skeleton; lands in M1)")
}
