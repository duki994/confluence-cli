//! High-level [`Auth`] façade.
//!
//! This is the API that `confluence-cli`'s command handlers call. It owns
//! a `Box<dyn AuthStore>` so the same handler code works against the
//! production keyring backend in `main` and against `MemoryStore` in
//! tests.

use crate::credential::{AuthMethod, Credential};
use crate::error::{Error, Result};
use crate::hosts::{validate_host, HostEntry, Hosts};
use crate::paths::Paths;
use crate::secret::SecretString;
use crate::store::{AuthStore, KeyringStore};

/// User-visible summary of one host's auth state. Returned by
/// [`Auth::status`]; rendered into a table/JSON by `confluence-cli`'s
/// output layer (this crate stays format-free).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub host: String,
    pub email: String,
    pub auth_method: AuthMethod,
    /// `true` if the keyring currently holds a secret for this host.
    /// Useful for diagnosing "metadata-without-secret" drift.
    pub token_present: bool,
    /// `true` if this host is the current default.
    pub is_default: bool,
}

/// Top-level auth API.
///
/// Construct one with [`Auth::production`] for normal use, or
/// [`Auth::with_store`] from a test passing in a [`crate::MemoryStore`].
pub struct Auth {
    store: Box<dyn AuthStore>,
}

impl std::fmt::Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Don't try to debug-print the store (might leak), don't expose
        // any internals; "{ .. }" is enough for diagnostics.
        f.debug_struct("Auth").finish_non_exhaustive()
    }
}

impl Auth {
    /// Wire up the production [`KeyringStore`] with the platform-resolved
    /// `hosts.toml` path.
    pub fn production() -> Result<Self> {
        let paths = Paths::discover()?;
        let hosts = Hosts::open(paths.hosts_file());
        Ok(Self::with_store(Box::new(KeyringStore::new(hosts))))
    }

    /// Inject any [`AuthStore`] — used by tests and by callers that want
    /// to swap in a custom backend.
    pub fn with_store(store: Box<dyn AuthStore>) -> Self {
        Self { store }
    }

    /// Register or update a host's credential. The host becomes the
    /// default automatically if no default was set.
    pub fn login(&self, host: &str, email: &str, token: SecretString) -> Result<()> {
        validate_host(host)?;
        let cred = Credential::api_token(email, token);
        self.store.put(host, email, cred)
    }

    /// Forget the credential and metadata for `host`.
    pub fn logout(&self, host: &str) -> Result<()> {
        validate_host(host)?;
        self.store.remove(host)
    }

    /// All registered hosts and their metadata. Stable order (`BTreeMap`).
    pub fn list(&self) -> Result<Vec<(String, HostEntry)>> {
        self.store.list_hosts()
    }

    /// Switch the active default host.
    pub fn switch(&self, host: &str) -> Result<()> {
        validate_host(host)?;
        self.store.set_default_host(host)
    }

    /// The currently active host and its metadata, if any.
    pub fn active(&self) -> Result<Option<(String, HostEntry)>> {
        let Some(host) = self.store.default_host()? else {
            return Ok(None);
        };
        let hosts = self.store.list_hosts()?;
        let entry = hosts
            .into_iter()
            .find(|(h, _)| h == &host)
            .map(|(_, e)| e)
            .ok_or_else(|| Error::HostNotFound(host.clone()))?;
        Ok(Some((host, entry)))
    }

    /// Fetch the credential for a host. Returns [`Error::HostNotFound`]
    /// if there is no entry.
    pub fn credential(&self, host: &str) -> Result<Credential> {
        validate_host(host)?;
        self.store.get(host)
    }

    /// Build a non-secret summary suitable for rendering.
    ///
    /// If `host` is `None`, the default host is used and the call fails
    /// with [`Error::NoActiveHost`] if no default is set.
    pub fn status(&self, host: Option<&str>) -> Result<Status> {
        let default = self.store.default_host()?;
        let resolved = match host {
            Some(h) => {
                validate_host(h)?;
                h.to_owned()
            }
            None => default.clone().ok_or(Error::NoActiveHost)?,
        };
        let hosts = self.store.list_hosts()?;
        let entry = hosts
            .into_iter()
            .find(|(h, _)| h == &resolved)
            .map(|(_, e)| e)
            .ok_or_else(|| Error::HostNotFound(resolved.clone()))?;
        // `token_present` checks the keyring, not just metadata, so a
        // half-broken state (metadata row but missing secret) is visible.
        let token_present = self.store.get(&resolved).is_ok();
        Ok(Status {
            is_default: default.as_deref() == Some(resolved.as_str()),
            host: resolved,
            email: entry.email,
            auth_method: entry.auth_method,
            token_present,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Auth;
    use crate::credential::AuthMethod;
    use crate::error::Error;
    use crate::secret::SecretString;
    use crate::store::MemoryStore;

    fn fresh() -> Auth {
        Auth::with_store(Box::new(MemoryStore::new()))
    }

    #[test]
    fn login_then_list_returns_one_entry() {
        let a = fresh();
        a.login("h.example", "u@e.test", SecretString::new("tok".to_owned()))
            .unwrap();
        let hosts = a.list().unwrap();
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].0, "h.example");
        assert_eq!(hosts[0].1.email, "u@e.test");
        assert_eq!(hosts[0].1.auth_method, AuthMethod::ApiToken);
    }

    #[test]
    fn first_login_becomes_default_then_switch_changes_it() {
        let a = fresh();
        a.login("a.example", "u@e.test", SecretString::new("1".to_owned()))
            .unwrap();
        a.login("b.example", "u@e.test", SecretString::new("2".to_owned()))
            .unwrap();
        // First login becomes default; second login does not steal it.
        let active = a.active().unwrap().unwrap();
        assert_eq!(active.0, "a.example");
        a.switch("b.example").unwrap();
        assert_eq!(a.active().unwrap().unwrap().0, "b.example");
    }

    #[test]
    fn logout_removes_and_reassigns_default() {
        let a = fresh();
        a.login("a.example", "u@e.test", SecretString::new("1".to_owned()))
            .unwrap();
        a.login("b.example", "u@e.test", SecretString::new("2".to_owned()))
            .unwrap();
        a.logout("a.example").unwrap();
        let hosts = a.list().unwrap();
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].0, "b.example");
        // Default rolls to the remaining host since `a.example` was active.
        assert_eq!(a.active().unwrap().unwrap().0, "b.example");
    }

    #[test]
    fn status_without_args_uses_default() {
        let a = fresh();
        a.login("h.example", "u@e.test", SecretString::new("tok".to_owned()))
            .unwrap();
        let s = a.status(None).unwrap();
        assert_eq!(s.host, "h.example");
        assert_eq!(s.email, "u@e.test");
        assert!(s.token_present);
        assert!(s.is_default);
    }

    #[test]
    fn status_with_no_hosts_is_no_active_host() {
        let a = fresh();
        assert!(matches!(a.status(None), Err(Error::NoActiveHost)));
    }

    #[test]
    fn status_for_unknown_host_is_host_not_found() {
        let a = fresh();
        a.login("h.example", "u@e.test", SecretString::new("tok".to_owned()))
            .unwrap();
        assert!(matches!(
            a.status(Some("nope.example")),
            Err(Error::HostNotFound(_))
        ));
    }

    #[test]
    fn switch_to_unknown_host_is_host_not_found() {
        let a = fresh();
        a.login("h.example", "u@e.test", SecretString::new("tok".to_owned()))
            .unwrap();
        assert!(matches!(
            a.switch("nope.example"),
            Err(Error::HostNotFound(_))
        ));
    }

    #[test]
    fn invalid_host_string_is_rejected_before_storage() {
        let a = fresh();
        assert!(matches!(
            a.login(
                "https://x.example",
                "u@e.test",
                SecretString::new("t".to_owned())
            ),
            Err(Error::InvalidHost(_))
        ));
    }

    #[test]
    fn credential_round_trips_via_auth() {
        let a = fresh();
        a.login(
            "h.example",
            "u@e.test",
            SecretString::new("supertok".to_owned()),
        )
        .unwrap();
        let c = a.credential("h.example").unwrap();
        match c {
            crate::credential::Credential::ApiToken { token, .. } => {
                assert_eq!(token.expose(), "supertok");
            }
        }
    }
}
