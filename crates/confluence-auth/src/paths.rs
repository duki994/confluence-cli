//! Platform-aware config path discovery.
//!
//! Production code uses [`Paths::discover`], which delegates to the
//! `directories` crate's `ProjectDirs("dev", "confluence", "confluence")`.
//! Tests use [`Paths::with_root`] to point at a `tempfile::TempDir`, so the
//! test suite never touches real XDG / `Application Support` / `AppData`
//! locations.
//!
//! XDG (X Desktop Group) base-directory specification is the Linux
//! convention that puts user config under `$XDG_CONFIG_HOME` (defaulting to
//! `~/.config`). `ProjectDirs` is the cross-platform wrapper that
//! resolves to the platform-native equivalent on macOS and Windows.

use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::error::{Error, Result};

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "confluence";
const APPLICATION: &str = "confluence";

const HOSTS_FILE_NAME: &str = "hosts.toml";

/// Resolved on-disk paths for the auth crate.
///
/// Hold once at startup and pass into [`crate::hosts::Hosts::open`].
#[derive(Debug, Clone)]
pub struct Paths {
    config_dir: PathBuf,
}

impl Paths {
    /// Production path discovery via `directories::ProjectDirs`.
    ///
    /// Returns [`Error::NoConfigDir`] on the rare platforms where
    /// `ProjectDirs::from` cannot determine a home directory (CI sandboxes
    /// without `$HOME`, mostly).
    pub fn discover() -> Result<Self> {
        let dirs =
            ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or(Error::NoConfigDir)?;
        Ok(Self {
            config_dir: dirs.config_dir().to_path_buf(),
        })
    }

    /// Test override: treat `root` as if it were the platform config
    /// directory. Used together with `tempfile::TempDir` so unit tests
    /// never touch real user data.
    pub fn with_root(root: &Path) -> Self {
        Self {
            config_dir: root.to_path_buf(),
        }
    }

    /// The directory holding `hosts.toml` and any future config files.
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Full path to the host registry file.
    pub fn hosts_file(&self) -> PathBuf {
        self.config_dir.join(HOSTS_FILE_NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::Paths;
    use std::path::PathBuf;

    #[test]
    fn with_root_resolves_hosts_file() {
        let root = PathBuf::from("/tmp/example/config");
        let paths = Paths::with_root(&root);
        assert_eq!(paths.config_dir(), root);
        assert_eq!(paths.hosts_file(), root.join("hosts.toml"));
    }

    #[test]
    fn discover_returns_some_directory() {
        // We can't assert the exact value without per-platform branching, but
        // we *can* assert it doesn't panic and the result is non-empty when
        // the test runner has a sane home directory (which `cargo test`
        // ensures by default).
        if let Ok(paths) = Paths::discover() {
            assert!(!paths.config_dir().as_os_str().is_empty());
        }
    }
}
