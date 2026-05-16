# 9. `hosts.toml`: locked, atomic, ordered writes

Date: 2026-05-16

## Status

Accepted

## Context

`hosts.toml` is the only writable config file shipped by v0.1. It tracks
every Atlassian host the user has logged into (email, auth-method tag,
creation timestamp) plus which one is currently the default. Secrets live
in the OS keyring; this file only carries cleartext metadata.

Three failure modes apply to a writable user-config file:

1. **Concurrent edits.** Two `confluence auth login` invocations from
   parallel terminals (or from a script that fans out) both: read the
   file, mutate in memory, write back. Whichever writes last silently
   loses the other's row. This is a classic read-modify-write race.
2. **Crash atomicity.** A naive in-place write can be interrupted mid-way
   — by a power loss, a SIGKILL, a filesystem timeout — leaving a
   truncated or empty TOML. The next CLI invocation fails to parse it and
   the user loses access to every recorded host.
3. **Diff churn.** Users routinely sync dotfiles. A `HashMap`-backed
   serializer would emit host keys in a different order on every save,
   producing spurious diffs that obscure real changes.

We commit to fixing all three in v0.1.

## Decision

The file is wrapped by two types in
`crates/confluence-auth/src/hosts.rs`:

- **`HostsFile`** — the serde shape. Knows the schema, not the path. Can
  round-trip through TOML in tests without filesystem.
- **`Hosts`** — the filesystem-bound wrapper. Knows the path, owns the
  read-modify-write helper.

The mutation entry point is `Hosts::with_locked_file<F, T>(F)`. It:

1. Creates the parent directory if needed.
2. Opens (or creates) the file with read+write+create+no-truncate.
3. Takes an **exclusive OS-level file lock** via
   `fs2::FileExt::lock_exclusive`. The lock is released by an RAII
   `LockGuard` whose `Drop` calls `FileExt::unlock`, so every exit path
   — including `?` — releases.
4. Reads the current contents into a `HostsFile` (empty file → default).
5. Hands the `HostsFile` to the caller's closure.
6. If the closure returns `Ok` **and** mutated the snapshot, writes
   atomically to a tmp sibling, calls `sync_all`, then `rename`s over the
   target. The closure can return `Ok` without mutating to indicate the
   write is unneeded (e.g. `switch` to the host that is already default).

Two further decisions ride along inside this same ADR because they are
load-bearing for the same three failure modes:

- **`HostsFile::hosts` is `BTreeMap<String, HostEntry>`**, not `HashMap`.
  TOML emission walks the map in iteration order; `BTreeMap` iterates in
  sorted key order, so the file is byte-stable across saves regardless
  of insertion order.
- **`validate_host` runs inside the locked closure** (called from
  `Auth::login` / `switch` / `logout` before mutating the snapshot), so
  the lock also serializes input validation. A racing process cannot
  push partially-validated state.

## Consequences

- Two concurrent CLI processes editing `hosts.toml` block on the lock
  rather than corrupting each other. Effective behaviour is "whoever
  takes the lock first wins; the other one waits, then sees the first
  one's result."
- A crash between `f.write_all` and `fs::rename` leaves both the original
  `hosts.toml` and a stray `hosts.toml.tmp`. The original is intact. The
  next successful login overwrites the tmp.
- A crash after `rename` but before `sync_all` is precluded — the order
  is `write_all` → `sync_all` → `drop(f)` → `rename`. On ext4 default
  options this matters: without `sync_all`, the OS may durably commit
  the rename and lose the file contents on power loss.
- The serialized file dotfile-tracks cleanly: alphabetised host blocks,
  deterministic emission, no spurious diffs.
- `BTreeMap` is `O(log n)` vs `HashMap`'s amortised `O(1)`. For "number
  of Atlassian hosts a user has registered" (a single-digit count in
  practice) the cost is invisible. Switching to `HashMap` "for speed"
  would be a regression in the human-facing property without buying
  measurable performance.
- The `Hosts` / `HostsFile` split is the seam that lets the proptest
  round-trip suite exercise the serde shape without bringing a
  filesystem into the test (see `crates/confluence-auth/tests/hosts_proptest.rs`).
- The pattern (`with_locked_file` + tmp/fsync/rename + `BTreeMap`) is
  the template for any future writable config file in this crate. Reach
  for an in-process `RwLock` only when the resource is *not* shared
  across processes — for `hosts.toml`, it always is.
- The lock is **advisory**: a non-cooperating process can still stomp
  the file. All `confluence-auth` mutation paths cooperate; outside
  editors are out of scope.
