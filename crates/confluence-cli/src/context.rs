//! Per-invocation context shared across command handlers.
//!
//! M0 stub: holds only the parsed global flags. The API client and auth
//! store accessors land alongside their respective crates in M1.

use crate::output::OutputFormat;

#[derive(Debug)]
#[allow(dead_code)] // Fields wired into handlers from M1 onward.
pub struct Context {
    pub host: Option<String>,
    pub output: OutputFormat,
    pub verbose: u8,
    pub no_color: bool,
}

impl Context {
    pub fn new(host: Option<String>, output: OutputFormat, verbose: u8, no_color: bool) -> Self {
        Self {
            host,
            output,
            verbose,
            no_color,
        }
    }
}
