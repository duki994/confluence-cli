# Architecture Decision Records

This directory holds the project's ADRs (Architecture Decision Records),
following Michael Nygard's
[format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
and managed with [adr-tools](https://github.com/npryce/adr-tools).

The location is configured at the repository root in `.adr-dir`, so
`adr new "..."`, `adr link`, and `adr generate toc` from anywhere in the
repo create files here.

* [1. Record architecture decisions](0001-record-architecture-decisions.md)
* [2. Cloud only for v0.1](0002-cloud-only-for-v0-1.md)
* [3. API-token-only authentication for v0.1](0003-api-token-only-authentication-for-v0-1.md)
* [4. Three-crate workspace layout](0004-three-crate-workspace-layout.md)
* [5. Storage format only for v0.1](0005-storage-format-only-for-v0-1.md)
* [6. Clippy pedantic with documented carve-outs](0006-clippy-pedantic-with-documented-carve-outs.md)
* [7. Secrets wrapped in a zeroizing newtype](0007-secrets-wrapped-in-a-zeroizing-newtype.md)
* [8. `#[non_exhaustive]` on public enums for forward-compat](0008-non-exhaustive-on-public-enums-for-forward-compat.md)
* [9. `hosts.toml`: locked, atomic, ordered writes](0009-hosts-toml-locked-atomic-ordered-writes.md)
* [10. `AuthStore` trait object with `Send + Sync`](0010-authstore-trait-object-with-send-and-sync.md)

## Authoring

```sh
# create a new ADR
adr new "Title of the decision"

# create one that supersedes 0003
adr new -s 3 "Replacement for API-token-only auth"

# link two ADRs (e.g. "Amends 0002")
adr link 7 Amends 2 "Amended by"
```

## Regenerating this index

Both this file and the ADR section of `Docs/src/SUMMARY.md` are derived from
`adr generate toc`. After adding, renaming, or removing an ADR, run:

```sh
# This file (mdBook landing page for ADRs)
adr generate toc \
    -i Docs/adr-fragments/_intro.md \
    -o Docs/adr-fragments/_outro.md \
    > Docs/src/adr/README.md

# The ADR sub-tree in SUMMARY.md (regenerated entries, transformed for
# mdBook indentation under the "Architecture" section)
adr generate toc -p adr/ | tail -n +3 \
    | sed -E 's|^\* |  - |; s|\(adr/|(./adr/|'
```

The second command emits the bullets to splice into `SUMMARY.md`; the rest of
that file is hand-maintained.

## Status conventions

- **Proposed** — under discussion, not yet in force.
- **Accepted** — current. Do not edit the body; supersede with a new ADR
  if the decision needs to change.
- **Superseded by NNNN** — replaced by a later ADR; kept for history.
- **Deprecated** — no longer in force, with no direct successor.
