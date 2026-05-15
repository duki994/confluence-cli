# Confluence CLI

`confluence` is a command-line tool for Atlassian Confluence, modeled on style of
[`gh`](https://cli.github.com/). It gives you ergonomic terminal access to
pages, spaces, search, attachments, and comments — with sensible defaults,
machine-readable output for scripting, and safe write semantics.

## Status

This documentation tracks the **v0.1** roadmap. v0.1 is intentionally narrow:

- **Cloud only.** Atlassian Server / Data Center support is planned but not
  yet present.
- **API-token authentication only.** OAuth and PAT are planned for later.
- **Confluence storage format only.** Markdown ↔ storage conversion is on
  the roadmap; in v0.1 you supply storage XHTML directly.

The current milestone is **M0** — workspace skeleton, command surface, CI,
and these docs. No commands actually do anything yet; they all return a
`not yet implemented` error. Subsequent milestones fill in the bodies:

| Milestone | Scope |
|-----------|----------------------------------------------------------|
| M0 | Skeleton, CI, docs (you are here) |
| M1 | Cloud auth, generic `api` request, request infrastructure |
| M2 | Read-only `page`, `space`, and `search` |
| M3 | Write paths for `page` (create, edit, delete) with safe version handling |
| M4 | Attachments, comments, completions, man pages |
| M5 | First tagged release with cross-platform binaries |

See the project plan for the full roadmap, including post-1.0 ideas.
