# 1. Record architecture decisions

Date: 2026-05-16

## Status

Accepted

## Context

We need to record the architectural decisions made on this project.

## Decision

We will use Architecture Decision Records, as
[described by Michael Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions).

ADRs live in `Docs/src/adr/`. The location is configured in `.adr-dir` at the
repository root so [adr-tools](https://github.com/npryce/adr-tools) (`adr new`,
`adr link`, `adr generate toc`) picks it up automatically.

Each ADR captures one decision, in Nygard's format: **Context**, **Decision**,
**Consequences**. ADRs are immutable once **Accepted** — to change a decision,
add a new ADR that **Supersedes** the previous one. Statuses we use:

- `Proposed` — under discussion.
- `Accepted` — current.
- `Superseded by NNNN` — replaced by a later ADR.
- `Deprecated` — no longer in force, with no direct successor.

## Consequences

- `CLAUDE.md`, the README, and the mdBook hold the *rules*; ADRs hold the
  *reasoning*. When the two disagree, the ADR is the source of truth and
  the other should be updated.
- ADRs are part of the mdBook (`Docs/src/adr/`) so they are published to
  GitHub Pages alongside the rest of the docs.
- See Nygard's article (linked above) and Nat Pryce's
  [adr-tools](https://github.com/npryce/adr-tools) for the toolset.
