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
