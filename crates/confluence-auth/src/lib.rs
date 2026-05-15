//! Credential storage and host registry for the `confluence` CLI.
//!
//! This crate owns the on-disk config (`hosts.toml`, `config.toml`) and the
//! OS keyring integration that holds the actual secret material. It does
//! **not** make HTTP requests or handle CLI argument parsing — see
//! `confluence-api` and `confluence-cli` respectively.
//!
//! M0 status: surface only. The `Credential` enum, `AuthStore` trait, and
//! keyring/file-backed implementations land in M1. See
//! `.claude/agents/confluence-auth-rust-developer.md` for the design contract.

use thiserror::Error;

/// Errors produced by the auth/storage layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Placeholder until real variants are introduced in M1.
    #[error("confluence-auth: not yet implemented")]
    NotImplemented,
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
