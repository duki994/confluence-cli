//! Credential storage and host registry for the `confluence` CLI.
//!
//! This crate owns:
//!
//! - The [`Credential`] enum and the redacted [`SecretString`] wrapper.
//! - The [`AuthStore`] trait and its concrete implementations
//!   (production [`KeyringStore`](crate::store::KeyringStore) backed by
//!   the OS keyring, [`MemoryStore`] for tests).
//! - The on-disk host registry (`hosts.toml`), atomic-write +
//!   `fs2`-locked.
//! - Cross-platform path discovery via [`Paths`].
//!
//! What it does **not** do (out of scope by design; see ADR 0004):
//!
//! - HTTP, retries, or REST — those live in `confluence-api`.
//! - Token validation against the live Confluence API — the CLI calls
//!   `confluence-api` first and then [`Auth::login`] on success.
//! - Interactive prompting for tokens or passphrases — `confluence-cli`.
//!
//! ## Quick start
//!
//! ```no_run
//! use confluence_auth::{Auth, SecretString};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let auth = Auth::production()?;
//! auth.login(
//!     "your-org.atlassian.net",
//!     "alice@example.com",
//!     SecretString::new(std::env::var("CONFLUENCE_API_TOKEN")?),
//! )?;
//! # Ok(()) }
//! ```

mod auth;
mod credential;
mod error;
mod hosts;
mod paths;
mod secret;
mod store;

pub use crate::auth::{Auth, Status};
pub use crate::credential::{AuthMethod, Credential};
pub use crate::error::{Error, Result};
pub use crate::hosts::HostEntry;
pub use crate::paths::Paths;
pub use crate::secret::SecretString;
pub use crate::store::{AuthStore, MemoryStore};
