# 10. `AuthStore` trait object with `Send + Sync`

Date: 2026-05-16

## Status

Accepted

## Context

`confluence-auth` exposes a single high-level façade, `Auth`, used by
every command handler in `confluence-cli`. The crate needs to swap the
credential storage backend in three settings:

- **Production:** an OS-keyring-backed `KeyringStore` paired with
  [[0009-hosts-toml-locked-atomic-ordered-writes]].
- **Unit and integration tests:** an in-process `MemoryStore` with no
  filesystem and no OS-keyring quirks. The keyring crate's mock builder
  hands out a fresh `MockCredential` per `Entry::new` call, so it cannot
  stand in for behavioural round-trip tests; only `MemoryStore` can.
- **Future encrypted-file fallback:** `encrypted_file::Store`, gated by
  the `fallback-file` Cargo feature for headless Linux without an OS
  keyring. The seam is already cut (`crates/confluence-auth/src/store/encrypted_file.rs`,
  inactive under default features).

There are two natural Rust implementations:

- **Generic dispatch:** `Auth<S: AuthStore>`. Zero-cost, but every
  consumer of `Auth` picks up the type parameter — the binary's `main`,
  every command handler signature, every test helper.
- **Trait-object dispatch:** `Auth { store: Box<dyn AuthStore> }`.
  Concrete `Auth`, one vtable indirection per store call.

## Decision

Use a trait object. The `Auth` struct holds a single
`Box<dyn AuthStore>` field (`crates/confluence-auth/src/auth.rs`):

```rust
pub struct Auth {
    store: Box<dyn AuthStore>,
}
```

The trait is bounded `pub trait AuthStore: Send + Sync`
(`crates/confluence-auth/src/store/mod.rs`). Concrete implementations:

- `MemoryStore` — `pub`, re-exported from the crate root for tests.
- `KeyringStore` — `pub(crate)`. Production callers reach it via
  `Auth::production()`; the choice of production backend stays an
  implementation detail.
- `encrypted_file::Store` — planned, `pub(crate)`, behind
  `#[cfg(feature = "fallback-file")]`.

`Box`, not `Arc`: `Auth` owns its store exclusively. If multiple owners
are ever needed (e.g. tokio task fan-out from a long-running daemon
mode), the right wrap is `Arc<Auth>`, not `Arc<dyn AuthStore>`.

## Consequences

- Command handlers, tests, and the binary all hold a plain `Auth` with
  no type parameter rippling through the codebase. A handler signature
  stays `pub async fn run(args, ctx) -> Result<()>`
  (see [[0006-clippy-pedantic-with-documented-carve-outs]] for why that
  signature is uniform).
- Tests swap backends at construction:
  `Auth::with_store(Box::new(MemoryStore::new()))`.
- Each `AuthStore` method call pays one vtable indirection. For a CLI
  that issues a handful of operations per invocation, the cost is
  invisible. Generic dispatch would not buy a measurable win here.
- The trait must remain **object-safe**: no methods returning `Self`,
  no methods taking `Self` by value, no generic methods. The current
  `AuthStore` surface satisfies that. New methods must too — adding a
  generic method later would silently fail to compile via `dyn`.
- `Send + Sync` is currently unexercised — the CLI is single-threaded
  per invocation. The bound is cheap to keep and forward-compatible
  with future tokio fan-out. It also rules out accidental `Rc<...>` /
  `RefCell<...>` inside a backend.
- The encrypted-file fallback (`fallback-file` feature) lands as a third
  `AuthStore` impl with no public-API change. The seam is the trait, not
  a feature-gated variant of `Auth`. Users opt in by building with the
  feature; everything downstream is identical.
- Cross-link: variants of `Credential` and `Error` flowing through this
  trait stay forward-compatible thanks to
  [[0008-non-exhaustive-on-public-enums-for-forward-compat]].
