//! Production [`AuthStore`] backed by the OS keyring plus an on-disk
//! [`Hosts`] registry.
//!
//! - Secrets (`SecretString`) go through [`keyring::Entry`] under the
//!   service name `"confluence-cli"` and account name `<host>`.
//! - Non-secret metadata (`email`, `auth_method`, `created_at`, the active
//!   default host) lives in `hosts.toml`, managed by [`Hosts`].
//!
//! Errors from the `keyring` crate are mapped at this layer into the
//! crate's typed [`Error`] enum so the trait surface stays clean.

use chrono::Utc;
use keyring::Entry;

use crate::credential::{AuthMethod, Credential};
use crate::error::{Error, Result};
use crate::hosts::{validate_host, HostEntry, Hosts};
use crate::secret::SecretString;

use super::AuthStore;

/// Service name passed to every `keyring::Entry`. Matches the binary name.
const SERVICE: &str = "confluence-cli";

/// Production [`AuthStore`]. Not re-exported from the crate root; callers
/// obtain one indirectly via [`crate::Auth::production`].
#[derive(Debug)]
pub(crate) struct KeyringStore {
    hosts: Hosts,
}

impl KeyringStore {
    pub(crate) fn new(hosts: Hosts) -> Self {
        Self { hosts }
    }

    fn entry(host: &str) -> Result<Entry> {
        Entry::new(SERVICE, host).map_err(|source| classify_keyring_error(host, source))
    }
}

/// Map a `keyring::Error` into the crate's typed error. Distinguishes
/// "the host has no entry" (`NoEntry`) from "the platform has no usable
/// keyring service" (`PlatformFailure`, `NoStorageAccess`).
fn classify_keyring_error(host: &str, source: keyring::Error) -> Error {
    match source {
        keyring::Error::NoEntry => Error::HostNotFound(host.to_owned()),
        keyring::Error::PlatformFailure(ref e) | keyring::Error::NoStorageAccess(ref e) => {
            Error::KeyringUnavailable {
                message: e.to_string(),
            }
        }
        other => Error::Keyring {
            host: host.to_owned(),
            source: other,
        },
    }
}

impl AuthStore for KeyringStore {
    fn list_hosts(&self) -> Result<Vec<(String, HostEntry)>> {
        let file = self.hosts.load()?;
        Ok(file.hosts.into_iter().collect())
    }

    fn get(&self, host: &str) -> Result<Credential> {
        validate_host(host)?;
        let file = self.hosts.load()?;
        let meta = file
            .hosts
            .get(host)
            .ok_or_else(|| Error::HostNotFound(host.to_owned()))?
            .clone();
        let entry = Self::entry(host)?;
        let password = entry
            .get_password()
            .map_err(|source| classify_keyring_error(host, source))?;
        Ok(Credential::api_token(
            meta.email,
            SecretString::new(password),
        ))
    }

    fn put(&self, host: &str, email: &str, cred: Credential) -> Result<()> {
        validate_host(host)?;
        let Credential::ApiToken { token, .. } = cred;
        // Write the secret first; if it fails we won't have an
        // orphaned metadata row.
        let entry = Self::entry(host)?;
        entry
            .set_password(token.expose())
            .map_err(|source| classify_keyring_error(host, source))?;

        self.hosts.with_locked_file(|file| {
            let created_at = file.hosts.get(host).map_or_else(Utc::now, |e| e.created_at);
            file.hosts.insert(
                host.to_owned(),
                HostEntry {
                    email: email.to_owned(),
                    auth_method: AuthMethod::ApiToken,
                    created_at,
                },
            );
            if file.default.is_none() {
                file.default = Some(host.to_owned());
            }
            Ok(())
        })
    }

    fn remove(&self, host: &str) -> Result<()> {
        validate_host(host)?;
        // Confirm the metadata row exists first so we return a clean
        // `HostNotFound` rather than a `keyring::Error::NoEntry`.
        let file = self.hosts.load()?;
        if !file.hosts.contains_key(host) {
            return Err(Error::HostNotFound(host.to_owned()));
        }

        // Best effort delete from keyring. `NoEntry` is fine — the
        // metadata row is the source of truth.
        match Self::entry(host)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(source) => return Err(classify_keyring_error(host, source)),
        }

        self.hosts.with_locked_file(|file| {
            file.hosts.remove(host);
            if file.default.as_deref() == Some(host) {
                file.default = file.hosts.keys().next().cloned();
            }
            Ok(())
        })
    }

    fn default_host(&self) -> Result<Option<String>> {
        Ok(self.hosts.load()?.default)
    }

    fn set_default_host(&self, host: &str) -> Result<()> {
        validate_host(host)?;
        self.hosts.with_locked_file(|file| {
            if !file.hosts.contains_key(host) {
                return Err(Error::HostNotFound(host.to_owned()));
            }
            file.default = Some(host.to_owned());
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    //! These tests use `keyring::set_default_credential_builder` to
    //! install the in-process mock backend before each test, so they
    //! never touch the real OS keyring.
    //!
    //! The mock backend is process-global, so we put it behind a `OnceLock`
    //! and accept that ordering of mock setup vs. real-keyring smoke tests
    //! does not matter: this whole block is `cfg(test)`.

    use super::{classify_keyring_error, KeyringStore, SERVICE};
    use crate::credential::Credential;
    use crate::error::Error;
    use crate::hosts::Hosts;
    use crate::secret::SecretString;
    use crate::store::AuthStore;
    use std::sync::OnceLock;

    fn install_mock() {
        static ONCE: OnceLock<()> = OnceLock::new();
        ONCE.get_or_init(|| {
            keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        });
    }

    fn store(tmp: &tempfile::TempDir) -> KeyringStore {
        install_mock();
        KeyringStore::new(Hosts::open(tmp.path().join("hosts.toml")))
    }

    fn cred(email: &str, tok: &str) -> Credential {
        Credential::api_token(email, SecretString::new(tok.to_owned()))
    }

    #[test]
    fn classify_no_entry_maps_to_host_not_found() {
        let mapped = classify_keyring_error("h.example", keyring::Error::NoEntry);
        assert!(matches!(mapped, Error::HostNotFound(h) if h == "h.example"));
    }

    #[test]
    fn classify_platform_failure_maps_to_keyring_unavailable() {
        let inner: Box<dyn std::error::Error + Send + Sync> = "no DBus".into();
        let mapped = classify_keyring_error("h.example", keyring::Error::PlatformFailure(inner));
        assert!(matches!(mapped, Error::KeyringUnavailable { .. }));
    }

    #[test]
    fn put_persists_metadata_via_mock_backend() {
        // The mock backend creates a fresh `Credential` per `Entry::new`
        // (no persistence between handles), so we can't round-trip a
        // secret through it. We *can* verify that `put` writes the
        // non-secret metadata row to `hosts.toml` and assigns a default.
        // Behavioral round-trip coverage lives on `MemoryStore` tests.
        let tmp = tempfile::tempdir().unwrap();
        let s = store(&tmp);
        s.put("h.example", "u@e.test", cred("u@e.test", "secret-token"))
            .unwrap();
        let listed = s.list_hosts().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].0, "h.example");
        assert_eq!(listed[0].1.email, "u@e.test");
        assert_eq!(s.default_host().unwrap().as_deref(), Some("h.example"));
    }

    #[test]
    fn remove_unknown_returns_host_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(&tmp);
        assert!(matches!(
            s.remove("nope.example"),
            Err(Error::HostNotFound(_))
        ));
    }

    #[test]
    fn set_default_unknown_returns_host_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let s = store(&tmp);
        assert!(matches!(
            s.set_default_host("nope.example"),
            Err(Error::HostNotFound(_))
        ));
    }

    // The real-OS keyring smoke test is opt-in via env var and ignored by
    // default. It hits the actual platform backend so it only runs on a
    // workstation with Secret Service / Keychain / Credential Manager.
    #[test]
    #[ignore = "real keyring; set CONFLUENCE_TEST_KEYRING=1 to run"]
    fn real_keyring_round_trip() {
        if std::env::var("CONFLUENCE_TEST_KEYRING").as_deref() != Ok("1") {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        // Note: this test deliberately does NOT install the mock backend.
        let s = KeyringStore::new(Hosts::open(tmp.path().join("hosts.toml")));
        let host = "confluence-cli-test-host.example";
        let _ = s.remove(host); // best-effort cleanup
        s.put(host, "u@e.test", cred("u@e.test", "real-token"))
            .unwrap();
        let got = s.get(host).unwrap();
        match got {
            Credential::ApiToken { token, .. } => assert_eq!(token.expose(), "real-token"),
        }
        s.remove(host).unwrap();
    }

    #[test]
    fn service_constant_matches_binary_name() {
        assert_eq!(SERVICE, "confluence-cli");
    }
}
