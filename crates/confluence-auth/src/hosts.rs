//! On-disk host registry — `hosts.toml`.
//!
//! Holds the cleartext metadata for every host the user has logged into:
//! the email, the auth method tag (never the secret itself), and a
//! creation timestamp. The secret lives in the OS keyring.
//!
//! Concurrency model: the [`Hosts::with_locked_file`] helper takes an
//! *exclusive* OS-level file lock via `fs2` around the
//! read-modify-write window, then writes atomically via tmp file + fsync
//! + rename. That eliminates the "two `confluence auth login` invocations
//!   racing each other" failure mode.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};

use crate::credential::AuthMethod;
use crate::error::{Error, Result};

/// One row in `hosts.toml`. The single source of truth for which email is
/// active on a given host.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HostEntry {
    /// The user's Atlassian account email.
    pub email: String,
    /// What kind of credential is stored for this host.
    pub auth_method: AuthMethod,
    /// When the user first ran `confluence auth login` for this host.
    pub created_at: DateTime<Utc>,
}

/// The full deserialized contents of `hosts.toml`.
///
/// `BTreeMap` (not `HashMap`) so the serialized file is deterministic:
/// hosts always appear in lexicographic order. That makes diffs in the
/// user's dotfile-tracked config sane.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct HostsFile {
    /// The active host: what `confluence` commands default to when
    /// `--host` is not passed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Map from host string (e.g. `your-org.atlassian.net`) to its
    /// non-secret metadata.
    ///
    /// `#[serde(default)]` so an absent `[hosts.*]` block deserializes
    /// to an empty map rather than failing.
    #[serde(default)]
    pub hosts: BTreeMap<String, HostEntry>,
}

impl HostsFile {
    /// Parse a `hosts.toml` payload. Errors carry the path for diagnostics.
    pub fn from_toml(path: &Path, src: &str) -> Result<Self> {
        toml::from_str(src).map_err(|source| Error::ConfigParse {
            path: path.to_path_buf(),
            source: Box::new(source),
        })
    }

    /// Render to a `hosts.toml` payload.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| Error::ConfigSerialize(Box::new(e)))
    }
}

/// Filesystem-backed view of the host registry.
///
/// Constructed once at startup with the resolved path. Every mutating
/// operation goes through [`with_locked_file`](Hosts::with_locked_file)
/// so concurrent CLI invocations cannot stomp each other.
#[derive(Debug, Clone)]
pub struct Hosts {
    path: PathBuf,
}

impl Hosts {
    /// Build a `Hosts` rooted at the given `hosts.toml` path. Does not
    /// touch the filesystem — that happens lazily on read/write.
    pub fn open(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load the current file contents (or `default()` if the file does
    /// not exist yet). Used by read-only callers; mutations should go
    /// through [`with_locked_file`](Self::with_locked_file).
    pub fn load(&self) -> Result<HostsFile> {
        match fs::read_to_string(&self.path) {
            Ok(s) => HostsFile::from_toml(&self.path, &s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HostsFile::default()),
            Err(source) => Err(Error::ConfigIo {
                path: self.path.clone(),
                source,
            }),
        }
    }

    /// Run `f` against the host registry under an exclusive file lock.
    ///
    /// 1. Ensure the parent directory exists.
    /// 2. Open (creating if needed) the hosts file.
    /// 3. Take an exclusive `fs2` lock.
    /// 4. Read current contents, hand them to `f`.
    /// 5. If `f` mutates and returns `Ok`, write atomically via tmp file
    ///    + `fsync` + `rename`.
    /// 6. Drop the lock.
    ///
    /// The closure returns its own `Result<T>` so a caller can decide
    /// nothing actually needs saving (e.g. `switch` to a host that is
    /// already the default).
    pub fn with_locked_file<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut HostsFile) -> Result<T>,
    {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::ConfigIo {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        // Touch + open with read+write so we can lock the file regardless
        // of whether it pre-exists.
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path)
            .map_err(|source| Error::ConfigIo {
                path: self.path.clone(),
                source,
            })?;

        file.lock_exclusive().map_err(|source| Error::ConfigIo {
            path: self.path.clone(),
            source,
        })?;

        // Make sure we always release the lock on every exit path. We use
        // a small guard struct rather than a `match` ladder.
        let _guard = LockGuard { file: &file };

        let mut hosts = read_locked(&self.path, &file)?;
        let before = hosts.clone();
        let outcome = f(&mut hosts)?;
        if hosts != before {
            atomic_write(&self.path, &hosts)?;
        }
        Ok(outcome)
    }
}

/// Read the (already-locked) file into a `HostsFile`.
fn read_locked(path: &Path, mut file: &File) -> Result<HostsFile> {
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|source| Error::ConfigIo {
            path: path.to_path_buf(),
            source,
        })?;
    if buf.trim().is_empty() {
        return Ok(HostsFile::default());
    }
    HostsFile::from_toml(path, &buf)
}

/// Tmp file -> fsync -> rename. The fsync is what protects against a crash
/// after the rename: without it, the OS may have committed only the rename
/// metadata and left the tmp file's contents un-flushed.
fn atomic_write(target: &Path, contents: &HostsFile) -> Result<()> {
    let serialized = contents.to_toml()?;
    let tmp = tmp_sibling(target);

    let mut f = File::create(&tmp).map_err(|source| Error::ConfigIo {
        path: tmp.clone(),
        source,
    })?;
    f.write_all(serialized.as_bytes())
        .map_err(|source| Error::ConfigIo {
            path: tmp.clone(),
            source,
        })?;
    f.sync_all().map_err(|source| Error::ConfigIo {
        path: tmp.clone(),
        source,
    })?;
    drop(f);

    fs::rename(&tmp, target).map_err(|source| Error::ConfigIo {
        path: target.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn tmp_sibling(target: &Path) -> PathBuf {
    let mut name = target.file_name().map_or_else(
        || std::ffi::OsString::from(".hosts.toml.tmp"),
        std::ffi::OsString::from,
    );
    name.push(".tmp");
    target.with_file_name(name)
}

/// RAII guard that unlocks the file when dropped.
struct LockGuard<'a> {
    file: &'a File,
}

impl Drop for LockGuard<'_> {
    fn drop(&mut self) {
        // Failing to unlock is not actionable from `Drop`; the OS releases
        // the lock when the FD closes regardless. Swallow the error.
        let _ = FileExt::unlock(self.file);
    }
}

/// Validate a host string before we touch the filesystem or keyring with
/// it. Rules: non-empty, no whitespace, no `://`, no leading/trailing dot
/// or slash. Anything stricter belongs in `confluence-cli` (it can refuse
/// an obviously-bogus host before calling us).
pub fn validate_host(host: &str) -> Result<()> {
    let trimmed = host.trim();
    if trimmed.is_empty()
        || trimmed.contains(char::is_whitespace)
        || trimmed.contains("://")
        || trimmed.starts_with('.')
        || trimmed.ends_with('.')
        || trimmed.contains('/')
    {
        return Err(Error::InvalidHost(host.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_host, HostEntry, Hosts, HostsFile};
    use crate::credential::AuthMethod;
    use chrono::{TimeZone, Utc};
    use std::collections::BTreeMap;

    fn sample_file() -> HostsFile {
        let mut hosts = BTreeMap::new();
        hosts.insert(
            "your-org.atlassian.net".to_owned(),
            HostEntry {
                email: "alice@example.com".to_owned(),
                auth_method: AuthMethod::ApiToken,
                created_at: Utc.with_ymd_and_hms(2026, 5, 15, 12, 0, 0).unwrap(),
            },
        );
        HostsFile {
            default: Some("your-org.atlassian.net".to_owned()),
            hosts,
        }
    }

    #[test]
    fn round_trip_serde() {
        let f = sample_file();
        let s = f.to_toml().unwrap();
        let parsed = HostsFile::from_toml(std::path::Path::new("hosts.toml"), &s).unwrap();
        assert_eq!(parsed, f);
    }

    #[test]
    fn missing_file_loads_as_default() {
        let dir = tempfile::tempdir().unwrap();
        let hosts = Hosts::open(dir.path().join("hosts.toml"));
        let loaded = hosts.load().unwrap();
        assert!(loaded.hosts.is_empty());
        assert!(loaded.default.is_none());
    }

    #[test]
    fn locked_write_creates_parent_dirs_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("hosts.toml");
        let hosts = Hosts::open(path.clone());
        hosts
            .with_locked_file(|f| {
                f.default = Some("h.example".to_owned());
                f.hosts.insert(
                    "h.example".to_owned(),
                    HostEntry {
                        email: "u@e.test".to_owned(),
                        auth_method: AuthMethod::ApiToken,
                        created_at: Utc.with_ymd_and_hms(2026, 5, 15, 0, 0, 0).unwrap(),
                    },
                );
                Ok(())
            })
            .unwrap();
        assert!(path.exists(), "hosts.toml should be written");
        let reloaded = hosts.load().unwrap();
        assert_eq!(reloaded.default.as_deref(), Some("h.example"));
        assert_eq!(reloaded.hosts.len(), 1);
    }

    #[test]
    fn locked_write_skips_io_when_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hosts.toml");
        let hosts = Hosts::open(path.clone());
        // First write creates the file.
        hosts
            .with_locked_file(|f| {
                f.default = Some("h.example".to_owned());
                Ok(())
            })
            .unwrap();
        let mtime_before = std::fs::metadata(&path).unwrap().modified().unwrap();
        // Second invocation makes no change; mtime should not advance
        // (within filesystem resolution). Sleep a beat just in case.
        std::thread::sleep(std::time::Duration::from_millis(20));
        hosts.with_locked_file(|_| Ok(())).unwrap();
        let mtime_after = std::fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after);
    }

    #[test]
    fn validate_host_accepts_normal_input() {
        assert!(validate_host("your-org.atlassian.net").is_ok());
        assert!(validate_host("sub.host.example").is_ok());
    }

    #[test]
    fn validate_host_rejects_bad_input() {
        assert!(validate_host("").is_err());
        assert!(validate_host("  ").is_err());
        assert!(validate_host("https://x.example").is_err());
        assert!(validate_host("x.example/path").is_err());
        assert!(validate_host(".x.example").is_err());
        assert!(validate_host("x.example.").is_err());
        assert!(validate_host("x x.example").is_err());
    }
}
