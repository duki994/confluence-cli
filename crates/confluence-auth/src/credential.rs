//! The [`Credential`] enum (the in-memory secret) and the [`AuthMethod`]
//! discriminator (its on-disk shadow).
//!
//! `Credential` carries the actual token via [`SecretString`]; `AuthMethod`
//! is the tag we persist into `hosts.toml`. Splitting the two is what lets
//! us serialize host metadata without ever touching the secret. The crate
//! never derives `Serialize`/`Deserialize` for `Credential`.

use serde::{Deserialize, Serialize};

use crate::secret::SecretString;

/// All credential shapes the crate knows about.
///
/// `#[non_exhaustive]` is critical here: roadmap variants (`OAuth`, `Pat`)
/// will be added without bumping the major version. Note the deliberate
/// absence of `#[derive(Clone)]` — see `rust_decisions_learning.md`.
#[derive(Debug)]
#[non_exhaustive]
pub enum Credential {
    /// Atlassian Cloud API token paired with the user's email.
    ApiToken {
        /// The user's Atlassian account email. Not a secret on its own.
        email: String,
        /// The API token. Wiped on drop, never logged.
        token: SecretString,
    },
}

impl Credential {
    /// Convenience constructor; mostly used in tests and CLI handlers.
    pub fn api_token(email: impl Into<String>, token: SecretString) -> Self {
        Self::ApiToken {
            email: email.into(),
            token,
        }
    }

    /// The on-disk `auth_method` tag corresponding to this variant.
    pub fn method(&self) -> AuthMethod {
        match *self {
            Self::ApiToken { .. } => AuthMethod::ApiToken,
        }
    }

    /// Borrow the email address regardless of variant. Reserved for future
    /// `OAuth` / `Pat` variants that might carry it differently.
    pub fn email(&self) -> &str {
        match self {
            Self::ApiToken { email, .. } => email,
        }
    }
}

/// On-disk discriminator for what kind of credential is associated with a
/// host.
///
/// Stored in `hosts.toml` as a snake-case string (`"api_token"`). Future
/// variants land here in lockstep with [`Credential`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AuthMethod {
    /// Atlassian Cloud API token.
    ApiToken,
}

#[cfg(test)]
mod tests {
    use super::{AuthMethod, Credential};
    use crate::secret::SecretString;

    #[test]
    fn auth_method_serializes_snake_case() {
        let s = toml::to_string(&AuthMethodWrap {
            v: AuthMethod::ApiToken,
        })
        .unwrap();
        assert!(s.contains("v = \"api_token\""), "got: {s}");
    }

    #[test]
    fn credential_method_matches_variant() {
        let c = Credential::api_token("a@b.test", SecretString::new("x".to_owned()));
        assert_eq!(c.method(), AuthMethod::ApiToken);
        assert_eq!(c.email(), "a@b.test");
    }

    #[test]
    fn credential_debug_does_not_leak_token() {
        let c = Credential::api_token("a@b.test", SecretString::new("topsecret".to_owned()));
        let rendered = format!("{c:?}");
        assert!(!rendered.contains("topsecret"));
        assert!(rendered.contains("<redacted>"));
    }

    // Wrapper so we can serialize the enum standalone via `toml`, which
    // requires a top-level table.
    #[derive(serde::Serialize)]
    struct AuthMethodWrap {
        v: AuthMethod,
    }
}
