//! `confluence` — Command-line tool for Atlassian Confluence.
//!
//! M0 status: clap scaffolding only. Every command handler returns a
//! "not yet implemented" error. The shape of the surface is real and stable;
//! the body of each handler will be filled in starting with M1.

use anyhow::Result;
use clap::Parser;

mod commands;
mod context;
mod output;

use commands::Command;
use context::Context;
use output::OutputFormat;

/// `Command-line tool for Atlassian Confluence.
#[derive(Debug, Parser)]
#[command(
    name = "confluence",
    version,
    about = "A command-line tool for Atlassian Confluence",
    long_about = None,
    propagate_version = true,
)]
struct Cli {
    /// Confluence host (e.g. `your-org.atlassian.net`). Overrides the default
    /// host configured via `confluence auth login`.
    #[arg(long, global = true, env = "CONFLUENCE_HOST")]
    host: Option<String>,

    /// Emit JSON instead of human-readable output. Shorthand for
    /// `--output json`.
    #[arg(long, global = true, conflicts_with = "output")]
    json: bool,

    /// Output format.
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Human)]
    output: OutputFormat,

    /// Increase log verbosity (repeat for more, e.g. `-vv`).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Disable ANSI colors in output. Also honored: `NO_COLOR` env var.
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let format = if cli.json {
        OutputFormat::Json
    } else {
        cli.output
    };
    let ctx = Context::new(cli.host, format, cli.verbose, cli.no_color);

    commands::dispatch(cli.command, &ctx).await
}
