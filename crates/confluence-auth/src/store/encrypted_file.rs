//! Encrypted-file fallback for environments without an OS keyring.
//!
//! M1 ships **only the seam**. The real implementation (age-encrypted file
//! at `~/.local/share/confluence/secrets.age`, passphrase-derived key,
//! KDF choice) will be designed and built in a later milestone.
//!
//! Activated by `--features fallback-file`. With the feature on, this
//! module compiles (so the API surface is exercised by CI), but no public
//! type is wired into [`crate::Auth::production`] yet.

// TODO(M2+): design the fallback file format. Decisions still open:
//   * KDF: argon2 vs. scrypt.
//   * Passphrase UX: env var, prompt, agent.
//   * File location: `~/.local/share/confluence/secrets.age` (XDG_DATA_HOME).
//   * Rotation / re-key flow.
