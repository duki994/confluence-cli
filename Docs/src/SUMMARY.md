# Summary

[Introduction](./intro.md)

# Getting started

- [Installation](./installation.md)
- [Authentication](./auth.md)

# Reference

- [Commands](./commands/README.md)

# Architecture

- [Decision Records](./adr/README.md)
  - [1. Record architecture decisions](./adr/0001-record-architecture-decisions.md)
  - [2. Cloud only for v0.1](./adr/0002-cloud-only-for-v0-1.md)
  - [3. API-token-only authentication for v0.1](./adr/0003-api-token-only-authentication-for-v0-1.md)
  - [4. Three-crate workspace layout](./adr/0004-three-crate-workspace-layout.md)
  - [5. Storage format only for v0.1](./adr/0005-storage-format-only-for-v0-1.md)
  - [6. Clippy pedantic with documented carve-outs](./adr/0006-clippy-pedantic-with-documented-carve-outs.md)
  - [7. Secrets wrapped in a zeroizing newtype](./adr/0007-secrets-wrapped-in-a-zeroizing-newtype.md)
  - [8. `#[non_exhaustive]` on public enums for forward-compat](./adr/0008-non-exhaustive-on-public-enums-for-forward-compat.md)
  - [9. `hosts.toml`: locked, atomic, ordered writes](./adr/0009-hosts-toml-locked-atomic-ordered-writes.md)
  - [10. `AuthStore` trait object with `Send + Sync`](./adr/0010-authstore-trait-object-with-send-and-sync.md)
