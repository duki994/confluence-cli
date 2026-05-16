//! Credential-storage trait and concrete implementations.
//!
//! All consumers go through the [`AuthStore`] trait so the production keyring
//! backend can be swapped for [`MemoryStore`] in tests. The OS-specific
//! [`keyring::KeyringStore`] is intentionally **not** re-exported from the
//! crate root: production callers obtain it via `Auth::production()`, which
//! keeps the choice of backend an implementation detail.

mod memory;

pub(crate) mod keyring;

#[cfg(feature = "fallback-file")]
pub(crate) mod encrypted_file;

use crate::credential::Credential;
use crate::error::Result;
use crate::hosts::HostEntry;

pub(crate) use self::keyring::KeyringStore;
pub use memory::MemoryStore;

/// Storage backend for credentials and the host registry.
///
/// `Send + Sync` so a single `Auth` can be shared across threads (the CLI
/// is single-threaded today, but the bound is cheap to keep and forward
/// compatible with future tokio task fan-out).
pub trait AuthStore: Send + Sync {
    /// All registered hosts and their non-secret metadata.
    fn list_hosts(&self) -> Result<Vec<(String, HostEntry)>>;

    /// Retrieve the credential stored for `host`.
    ///
    /// Returns [`crate::Error::HostNotFound`] if the host has no entry.
    fn get(&self, host: &str) -> Result<Credential>;

    /// Persist a credential. Overwrites any existing entry for the same
    /// host. The `email` argument is the on-disk record; the email
    /// embedded in [`Credential::ApiToken`] must match.
    fn put(&self, host: &str, email: &str, cred: Credential) -> Result<()>;

    /// Delete the host entry and its secret. A no-op `Ok(())` for a host
    /// that was never registered would mask user typos; impls return
    /// [`crate::Error::HostNotFound`] in that case.
    fn remove(&self, host: &str) -> Result<()>;

    /// The active host, or `None` if no host has been logged into.
    fn default_host(&self) -> Result<Option<String>>;

    /// Set the active host. Errors if the host is not registered.
    fn set_default_host(&self, host: &str) -> Result<()>;
}
