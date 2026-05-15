//! Output format selection.
//!
//! M0 stub: enum only. The `Renderer` trait described in the rust-developer
//! subagent file lands alongside the first command that produces real output
//! (M2: `page view`).

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
    Tsv,
}
