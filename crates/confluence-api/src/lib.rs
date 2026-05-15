//! Typed Confluence Cloud REST client.
//!
//! This crate is the HTTP layer for the `confluence` CLI. It owns request
//! building, pagination, retry/backoff, and response deserialization. It does
//! **not** handle credential storage, terminal output, or CLI argument
//! parsing — see `confluence-auth` and `confluence-cli` respectively.
//!
//! M0 status: surface only. Real endpoints land in M1 (`api` escape hatch +
//! request infrastructure) and M2 (typed read-only endpoints). See
//! `.claude/agents/confluence-api-rust-developer.md` for the design contract.

use thiserror::Error;

/// Errors produced by the Confluence API client.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Placeholder until real variants are introduced in M1.
    #[error("confluence-api: not yet implemented")]
    NotImplemented,
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
