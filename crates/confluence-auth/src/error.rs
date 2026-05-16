//! Error type for the `confluence-auth` crate.
//!
//! All public functions return [`Result<T>`], which is [`std::result::Result`]
//! parameterised on this crate's [`Error`]. The variant payloads are designed
//! to be *log-safe*: no `SecretString`, no raw token, no Authorization header.
//! If a future variant needs to embed user input, it must redact it before it
//! reaches `Display`.

use std::path::PathBuf;

/// All errors produced by the auth/storage layer.
///
/// The enum is `#[non_exhaustive]` so we can add a variant (for example, a
/// future `OAuth { .. }` failure mode) without breaking external `match`
/// statements at the call site.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The requested host is not registered in `hosts.toml`.
    #[error("no credentials registered for host `{0}`")]
    HostNotFound(String),

    /// No `default` host is set and the caller did not specify one.
    #[error("no active host; run `confluence auth login` or `confluence auth switch <host>`")]
    NoActiveHost,

    /// The OS keyring backend reports the user has no usable keyring
    /// service (typical on headless Linux without GNOME Keyring/KWallet).
    #[error("OS keyring unavailable: {message}")]
    KeyringUnavailable {
        /// Best-effort diagnostic. Never contains secret material.
        message: String,
    },

    /// A keyring operation failed for a specific host.
    ///
    /// The host string is included for context; the underlying
    /// [`keyring::Error`] is preserved as the source via `thiserror`.
    #[error("keyring operation failed for host `{host}`")]
    Keyring {
        host: String,
        #[source]
        source: keyring::Error,
    },

    /// Read or write of an on-disk config file failed at the IO layer.
    #[error("failed to access config file `{path}`")]
    ConfigIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// A config file is present but does not parse as valid TOML for our
    /// schema.
    ///
    /// The `toml::de::Error` payload is boxed to keep `Error` (and any
    /// `Result<_, Error>`) small enough to satisfy `clippy::result_large_err`
    /// on Windows, where `toml::de::Error` is ~128 bytes.
    #[error("failed to parse config file `{path}`")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },

    /// Serializing the in-memory `HostsFile` back to TOML failed.
    /// Normally a bug rather than a user-facing condition.
    #[error("failed to serialize config file")]
    ConfigSerialize(#[source] Box<toml::ser::Error>),

    /// The supplied host string failed validation (empty, whitespace, scheme,
    /// etc).
    #[error("invalid host `{0}`: must be a non-empty hostname without scheme or path")]
    InvalidHost(String),

    /// `directories::ProjectDirs` returned `None` — we could not determine a
    /// platform config directory.
    #[error("could not determine a platform config directory for `confluence`")]
    NoConfigDir,

    /// Reserved for the future encrypted-file fallback. Carries a free-form
    /// diagnostic; **never** a secret.
    #[error("encrypted-file store error: {0}")]
    EncryptedFile(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
