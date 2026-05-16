# 8. `#[non_exhaustive]` on public enums for forward-compat

Date: 2026-05-16

## Status

Accepted

## Context

`confluence-auth` exposes three public enums today:

- `Credential` — the in-memory secret. Only variant: `ApiToken { .. }`.
- `AuthMethod` — the on-disk tag for which kind of credential a host has.
  Only variant: `ApiToken`.
- `Error` — the crate's error type, with one variant per failure mode.

Per [[0003-api-token-only-authentication-for-v0-1]] the credential shape
is intentionally narrow in v0.1: API tokens only. OAuth 2.0 (3LO) and PAT
are explicit roadmap items, not absent by accident. Per
[[0002-cloud-only-for-v0-1]] Server/Data Center support is also roadmap;
landing it will add error variants too (e.g. server-side health probes).

Adding a variant to a normal public enum is a **breaking change**.
Downstream code that `match`es exhaustively stops compiling when a new
arm appears, which forces a major version bump even though the existing
variants behave identically. We want the OAuth and PAT variants to land
inside the same major version — they are extensions, not redesigns.

## Decision

Every public enum in `confluence-auth` carries `#[non_exhaustive]`:

- `Error` — `crates/confluence-auth/src/error.rs`.
- `Credential` and `AuthMethod` — `crates/confluence-auth/src/credential.rs`.

The attribute forces external `match` blocks to include a wildcard arm,
which means we can add `Credential::OAuth { .. }`, `AuthMethod::OAuth`,
or `Error::OAuthRefreshFailed { .. }` without breaking any
already-compiled consumer code.

## Consequences

- The roadmap (`OAuth`, `Pat`, Server/DC error variants) lands as a minor
  bump, not a major bump.
- Downstream callers cannot exhaustively match on these enums — they must
  write `_ => ...` (or use a specific subset and accept the catch-all).
  That ergonomic cost is the explicit price of keeping the variant set
  open.
- The attribute is **not** load-bearing for crate-internal `match`
  expressions: `#[non_exhaustive]` is a hint to *downstream* crates only.
  Inside `confluence-auth` we still get full exhaustiveness checking, so
  adding a variant still surfaces every internal site that needs to
  handle it.
- When the variant set genuinely stabilises (likely no earlier than 1.0),
  per-enum revisits are appropriate — keep `#[non_exhaustive]` where new
  variants are still plausible, drop it where the design is closed.
- New public enums introduced in this crate should default to
  `#[non_exhaustive]` unless there is an explicit reason not to. A
  closed-set enum (a fixed mathematical domain, e.g. an HTTP method) is
  fine without it; anything tied to a feature roadmap should have it.
