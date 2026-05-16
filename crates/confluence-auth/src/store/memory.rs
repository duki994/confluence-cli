//! In-process [`AuthStore`] used by every command-handler test in
//! `confluence-cli`. Holds everything (secrets included) in a single
//! `Mutex<State>`. No filesystem, no keyring, no disk.

use std::collections::BTreeMap;
use std::sync::Mutex;

use chrono::Utc;

use crate::credential::Credential;
use crate::error::{Error, Result};
use crate::hosts::{validate_host, HostEntry};
use crate::secret::SecretString;

use super::AuthStore;

#[derive(Debug, Default)]
struct State {
    default: Option<String>,
    entries: BTreeMap<String, Entry>,
}

#[derive(Debug)]
struct Entry {
    meta: HostEntry,
    token: SecretString,
}

/// In-memory store for tests.
///
/// Cloning is intentionally not supported (the wrapped `Mutex` cannot
/// safely clone its locked state into a fresh instance without losing
/// invariants). Construct one per test.
#[derive(Debug, Default)]
pub struct MemoryStore {
    state: Mutex<State>,
}

impl MemoryStore {
    /// Build an empty store. Equivalent to `MemoryStore::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        // A `Mutex` poisons if a holder panics. We unwrap because every
        // test that hits this either succeeds or already failed for a
        // more interesting reason than poisoning.
        self.state.lock().expect("MemoryStore mutex poisoned")
    }
}

impl AuthStore for MemoryStore {
    fn list_hosts(&self) -> Result<Vec<(String, HostEntry)>> {
        let state = self.lock();
        Ok(state
            .entries
            .iter()
            .map(|(h, e)| (h.clone(), e.meta.clone()))
            .collect())
    }

    fn get(&self, host: &str) -> Result<Credential> {
        validate_host(host)?;
        let state = self.lock();
        let entry = state
            .entries
            .get(host)
            .ok_or_else(|| Error::HostNotFound(host.to_owned()))?;
        // Copy out the bytes; the original stays zeroized-on-drop. We can
        // not move out of the BTreeMap because the store still owns it.
        let token = SecretString::new(entry.token.expose().to_owned());
        Ok(Credential::api_token(entry.meta.email.clone(), token))
    }

    fn put(&self, host: &str, email: &str, cred: Credential) -> Result<()> {
        validate_host(host)?;
        let mut state = self.lock();
        let Credential::ApiToken { token, .. } = cred;
        let meta = HostEntry {
            email: email.to_owned(),
            auth_method: crate::credential::AuthMethod::ApiToken,
            created_at: state
                .entries
                .get(host)
                .map_or_else(Utc::now, |e| e.meta.created_at),
        };
        state.entries.insert(host.to_owned(), Entry { meta, token });
        if state.default.is_none() {
            state.default = Some(host.to_owned());
        }
        Ok(())
    }

    fn remove(&self, host: &str) -> Result<()> {
        validate_host(host)?;
        let mut state = self.lock();
        if state.entries.remove(host).is_none() {
            return Err(Error::HostNotFound(host.to_owned()));
        }
        if state.default.as_deref() == Some(host) {
            state.default = state.entries.keys().next().cloned();
        }
        Ok(())
    }

    fn default_host(&self) -> Result<Option<String>> {
        Ok(self.lock().default.clone())
    }

    fn set_default_host(&self, host: &str) -> Result<()> {
        validate_host(host)?;
        let mut state = self.lock();
        if !state.entries.contains_key(host) {
            return Err(Error::HostNotFound(host.to_owned()));
        }
        state.default = Some(host.to_owned());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryStore;
    use crate::credential::Credential;
    use crate::error::Error;
    use crate::secret::SecretString;
    use crate::store::AuthStore;

    fn cred(email: &str, tok: &str) -> Credential {
        Credential::api_token(email, SecretString::new(tok.to_owned()))
    }

    #[test]
    fn put_then_get_roundtrips() {
        let s = MemoryStore::new();
        s.put("h.example", "u@e.test", cred("u@e.test", "tok"))
            .unwrap();
        let got = s.get("h.example").unwrap();
        match got {
            Credential::ApiToken { email, token } => {
                assert_eq!(email, "u@e.test");
                assert_eq!(token.expose(), "tok");
            }
        }
    }

    #[test]
    fn first_put_becomes_default() {
        let s = MemoryStore::new();
        assert_eq!(s.default_host().unwrap(), None);
        s.put("h.example", "u@e.test", cred("u@e.test", "tok"))
            .unwrap();
        assert_eq!(s.default_host().unwrap().as_deref(), Some("h.example"));
    }

    #[test]
    fn remove_unknown_is_error() {
        let s = MemoryStore::new();
        assert!(matches!(s.remove("h.example"), Err(Error::HostNotFound(_))));
    }

    #[test]
    fn remove_default_picks_a_new_default() {
        let s = MemoryStore::new();
        s.put("a.example", "u@e.test", cred("u@e.test", "1"))
            .unwrap();
        s.put("b.example", "u@e.test", cred("u@e.test", "2"))
            .unwrap();
        s.set_default_host("b.example").unwrap();
        s.remove("b.example").unwrap();
        // We fall back to *some* remaining host.
        assert_eq!(s.default_host().unwrap().as_deref(), Some("a.example"));
    }

    #[test]
    fn set_default_unknown_is_error() {
        let s = MemoryStore::new();
        assert!(matches!(
            s.set_default_host("nope.example"),
            Err(Error::HostNotFound(_))
        ));
    }
}
