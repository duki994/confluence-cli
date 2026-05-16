# 4. Three-crate workspace layout

Date: 2026-05-16

## Status

Accepted

## Context

A single-crate CLI is faster to start but couples three concerns that have
genuinely different lifecycles and consumers:

- HTTP / REST client logic (request building, pagination, retry, version
  handling).
- Credential storage and host registry (OS keyring, config file, multi-host
  switching).
- The user-facing binary (clap parsing, output formatting, command dispatch,
  shell completions).

We also want the typed Confluence client to be usable as a Rust library, not
just an internal detail of `confluence`. Other tools — scripts, bots,
internal services — should be able to depend on it without dragging in
clap or keyring code.

## Decision

The repository is a Cargo workspace with three crates:

| Crate              | Responsibility                                                     |
|--------------------|--------------------------------------------------------------------|
| `confluence-api`   | Typed REST client. Pagination, retry/backoff, response decoding.   |
| `confluence-auth`  | Credential storage, config file (`hosts.toml`), host registry.     |
| `confluence-cli`   | clap binary, command handlers, output formatting, completions.    |

Dependency direction is one-way:

```
confluence-cli ──► confluence-api
confluence-cli ──► confluence-auth
```

`confluence-api` and `confluence-auth` do **not** depend on each other. The
CLI is the only place where credentials and HTTP requests meet.

## Consequences

- Cross-crate leakage is a bug, not a shortcut. Clap types in `confluence-api`,
  HTTP code in `confluence-auth`, terminal formatting anywhere but `confluence-cli`
  — all called out in `CLAUDE.md` and enforced in review.
- The `confluence-api` crate is structurally publishable to crates.io as a
  standalone library. We don't push the button at v0.1, but we keep the
  option open by not contaminating its public surface.
- More Cargo boilerplate (three `Cargo.toml`, workspace inheritance via
  `*.workspace = true`). Justified by the boundary clarity.
- Subagent routing mirrors the layout — `confluence-api-rust-developer`,
  `confluence-auth-rust-developer`, and the general `rust-developer`. The
  agent definitions in `.claude/agents/` are the contract.
- A new feature that spans two crates is coordinated at the workspace level
  (the CLI is the integration point), not by adding a back-edge between
  `confluence-api` and `confluence-auth`.
