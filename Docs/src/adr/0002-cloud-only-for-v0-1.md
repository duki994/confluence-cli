# 2. Cloud only for v0.1

Date: 2026-05-16

## Status

Accepted

## Context

Atlassian Confluence ships in two distinct product lines:

- **Confluence Cloud** — hosted at `*.atlassian.net`, REST v2 + REST v1, OAuth
  2.0 / API tokens / 3LO, rate limiting via headers, semantic versioning of
  the API surface.
- **Confluence Server / Data Center** — self-hosted, REST v1 only (no v2),
  Personal Access Tokens or basic auth, different pagination semantics, no
  guaranteed parity with Cloud endpoints.

Supporting both from day one would force us to:

- Maintain two HTTP request shapes and two response deserialization paths.
- Abstract over auth before we understand what the seams should look like.
- Test against two product matrices on every change.
- Make scope decisions (e.g. "does this endpoint exist on DC?") on a
  per-feature basis, slowing every PR.

We do not yet have a representative set of Server/DC users asking for this
tool, and the Cloud surface alone is large enough to fill several milestones.

## Decision

**v0.1 targets Confluence Cloud only.** The `confluence-api` crate may
hard-code Cloud-specific conventions (base path `/wiki/api/v2`, REST v1 fallback
for CQL, header-based rate limiting). Server / Data Center support is on the
post-1.0 roadmap.

## Consequences

- The `confluence-api` client stays small — no transport-level abstraction
  over Cloud vs. DC.
- We can rely on Confluence Cloud REST v2 idioms (cursor pagination, `If-Match`
  for optimistic concurrency, etc.) without conditional code paths.
- Server / DC users cannot use `confluence` in v0.1. README and `Docs/src/intro.md`
  call this out explicitly.
- When Server/DC support is added, the abstraction work and any breaking
  changes to the `confluence-api` crate will be captured by a superseding ADR.
- See also: [[0003-api-token-only-authentication-for-v0-1]] (the auth half of
  this same scope decision).
