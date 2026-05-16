# 3. API-token-only authentication for v0.1

Date: 2026-05-16

## Status

Accepted

## Context

Atlassian Cloud supports several authentication mechanisms, each with very
different ergonomics and implementation cost:

- **API tokens.** User generates a token at
  `id.atlassian.com/manage-profile/security/api-tokens`, the client uses HTTP
  Basic with `email:token`. Zero registration, no refresh, works for any
  workspace the user belongs to. Stored as one secret.
- **OAuth 2.0 (3LO).** Requires an Atlassian Connect / Forge app registration,
  consent flow with a browser callback, access + refresh tokens, scope
  management, and a token-refresh loop in the client.
- **Personal Access Tokens (PAT).** A Server / Data Center concept. Out of
  scope per [[0002-cloud-only-for-v0-1]].

The CLI's primary use cases — interactive terminal sessions, CI scripts —
fit API-token semantics naturally: one secret per host, no callback flow, no
refresh.

## Decision

**v0.1 supports API-token authentication only.** Credentials are stored via
the OS keyring through the `confluence-auth` crate; host metadata lives in
`hosts.toml` in the platform config directory. OAuth and PAT are on the
roadmap (post-1.0).

Path discovery for `hosts.toml` goes through `directories::ProjectDirs`
with the identifiers `("dev", "confluence", "confluence")`. That resolves
to `~/.config/confluence/hosts.toml` on Linux (XDG Base Directory spec),
`~/Library/Application Support/dev.confluence.confluence/hosts.toml` on
macOS, and `%APPDATA%\confluence\confluence\config\hosts.toml` on
Windows. Tests bypass discovery via `Paths::with_root(&tempdir)`, so the
suite never touches real user directories. The writable-file semantics
(advisory lock, tmp/fsync/rename, ordered emission) are documented
separately in [[0009-hosts-toml-locked-atomic-ordered-writes]].

## Consequences

- `confluence-auth` stays minimal: store/retrieve a secret, track which host
  is active, no refresh logic, no callback server, no app registration.
- Users self-serve tokens — no app registration, no admin involvement.
- CI/automation use cases work via the `CONFLUENCE_API_TOKEN` env var without
  any extra plumbing.
- When OAuth support lands, the credential store has to grow refresh handling
  and a per-credential type tag. That migration will be a superseding ADR.
- The CLI cannot act on behalf of an app or service principal in v0.1 — only
  as the user whose token is configured.
- See also: [[0002-cloud-only-for-v0-1]].
