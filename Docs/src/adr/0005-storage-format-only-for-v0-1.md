# 5. Storage format only for v0.1

Date: 2026-05-16

## Status

Accepted

## Context

Confluence pages can be represented in three formats, each with very different
implications for a CLI:

- **Confluence storage format.** An XHTML-based markup that the Confluence
  REST API accepts and emits directly. Round-trips losslessly. What the
  product stores internally.
- **Atlassian Document Format (ADF).** A JSON document model used by newer
  Atlassian editors. The REST v2 API can return ADF for some endpoints, but
  storage is still the canonical write format for Confluence pages.
- **Markdown.** Familiar to users, but not natively understood by Confluence.
  Conversion to/from storage involves a non-trivial mapping (tables,
  macros, links, emoji, anchors, mentions) and has many failure modes.
  `pandoc` and the various Rust converters each have edge cases.

Markdown ↔ storage conversion is, realistically, a serious project of its
own — large enough to be its own crate, with its own test corpus, its own
issue tracker for edge cases. Shipping it inside v0.1 would either delay
v0.1 or ship a half-working converter that users would have to reverse-engineer.

## Decision

**v0.1 accepts and emits Confluence storage format only.** Write commands
take storage XHTML on stdin or via `--file`. Read commands return storage
XHTML by default. ADF and Markdown conversion are on the roadmap (post-1.0).

## Consequences

- The `confluence-api` crate stays format-agnostic above the wire — it
  carries opaque bodies, doesn't try to parse or transform them.
- Power users (technical writers, automation scripts) get exact fidelity
  with no surprise conversions.
- Markdown-only users cannot author pages with `confluence` in v0.1 without
  pre-converting. README and `intro.md` call this out so expectations are
  set before install.
- We can revisit format support as a separate, focused effort — likely a
  new `confluence-convert` crate behind a feature flag on the CLI. That
  will be a separate ADR.
- See also: [[0002-cloud-only-for-v0-1]],
  [[0003-api-token-only-authentication-for-v0-1]] — the three v0.1 scope
  decisions are interdependent.
