//! `confluence search` — CQL search.
//!
//! M0 stub. Lands in M2. The CQL endpoint is REST v1.

use anyhow::{bail, Result};
use clap::Args as ClapArgs;

use crate::context::Context;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// CQL query string (e.g. `type=page AND space="ABC"`).
    cql: String,
    /// Maximum number of results to return.
    #[arg(long, default_value_t = 25)]
    limit: u32,
}

pub async fn run(_args: Args, _ctx: &Context) -> Result<()> {
    bail!("`search` is not yet implemented (M0 skeleton; lands in M2)")
}
