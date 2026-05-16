# 7. Secrets wrapped in a zeroizing newtype

Date: 2026-05-16

## Status

Accepted

## Context

`confluence-auth` handles long-lived Atlassian API tokens. Three default
Rust behaviours work against the security model the CLI promises:

- **`String` is not zeroized on drop.** When a `String` falls out of scope
  the allocator gets the heap buffer back without overwriting it. The token
  bytes sit in freed memory until something else reuses the page. A heap
  dump, a swapped-out page, or unsafe code can recover the value long after
  the variable is "gone."
- **`{:?}` / `{}` print the contents.** A stray `tracing::debug!("{cred:?}")`,
  a panic payload that captures the credential, or a derived `Debug` on a
  wrapping type leaks the token into logs.
- **`#[derive(Clone)]` silently multiplies secrets.** Each clone is another
  allocation the allocator can leak later; the `Drop` of one clone does
  nothing to the others.

CLAUDE.md already commits to the rule that tokens never appear in any log
at any level. That rule has to be enforced by the type system, not by
discipline — too many code paths touch a credential for "remember not to
print it" to be reliable.

## Decision

Wrap every in-memory token in a `SecretString` newtype around
[`zeroize::Zeroizing<String>`] (`crates/confluence-auth/src/secret.rs`):

- Public API is intentionally minimal: `SecretString::new(String)` to
  construct, `SecretString::expose(&self) -> &str` to read.
- `fmt::Debug` is implemented **by hand** and prints
  `SecretString(<redacted>)`. The derive macro is never used here.
- No `Clone`, no `Display`, no `serde::Serialize`, no
  `serde::Deserialize`. The crate also rejects them transitively: the
  `Credential` enum that carries the token derives `Debug` (which composes
  with our redacted impl) but deliberately does **not** derive `Clone` or
  any Serde traits.
- `Credential` itself is treated as a secret-bearing type. Its `Debug`
  format is verified by test (`credential_debug_does_not_leak_token`).

## Consequences

- Token bytes are overwritten by `Zeroizing<String>`'s `Drop` before the
  allocator gets the buffer back. A subsequent heap inspection cannot
  recover them.
- Any line that formats a `Credential` (or a `SecretString` directly) with
  `{:?}` is safe: the manual impl wins, even when the outer struct derives
  `Debug`.
- A token cannot accidentally enter a TOML / JSON serializer — there is no
  Serde impl to call.
- Callers cannot `clone()` a credential. The intended pattern is to
  re-fetch from the [[0010-authstore-trait-object-with-send-and-sync]]
  store. This is documented on the type and reflected in the tests.
- `expose()` returning `&str` (not `String`) keeps copies grep-able: any
  call site that needs an owned copy has to write `.expose().to_owned()`,
  which shows up in code review and audit logs.
- The wrapper is unergonomic on purpose. That ergonomic cost is the price
  of the security guarantee — there is no safe `Display`, so anyone
  printing a secret has to opt in by name.
- New credential shapes added under [[0008-non-exhaustive-on-public-enums-for-forward-compat]]
  must reuse `SecretString` for their secret-bearing fields. The CI lint
  for this is implicit (no `Clone` derive will compile through a
  `SecretString` field) plus the redaction test pattern.
