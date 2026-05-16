# Confluence CLI — Project Context

Binary name: `confluence`.

## Scope Discipline

v0.1 is intentionally narrow. **Do not expand scope without an explicit ask.**

- **Cloud only.** Server/Data Center support is roadmap, not present.
- **API-token auth only.** OAuth and PAT are roadmap.
- **Storage format only.** ADF and Markdown conversion are roadmap.

If a task implies any of the above, stop and confirm before implementing.
See `Docs/src/intro.md` and the project plan for the full milestone breakdown.

The *reasoning* behind these scope decisions lives in the ADRs under
`Docs/src/adr/` (0002 cloud-only, 0003 api-token-only, 0005 storage-format-only).
This section holds the rules; the ADRs hold the why. If the two disagree, the
ADR is the source of truth and this file should be updated.

## Repository Layout

```
crates/
  confluence-api/   # typed REST client; pagination, backoff, version handling
  confluence-auth/  # credential storage, config file, host registry
  confluence-cli/   # clap-driven binary; command handlers, output formatting
Docs/               # mdBook source, published to GH Pages
```

Each crate has a clear boundary. Cross-crate leakage (e.g. clap types in
`confluence-api`, HTTP code in `confluence-auth`) is a bug, not a shortcut.

## Subagent Routing

When a task is clearly within one crate's boundary, prefer the matching subagent:

- `confluence-api/` work → **confluence-api-rust-developer**
- `confluence-auth/` work → **confluence-auth-rust-developer**
- `confluence-cli/` work, or cross-cutting Rust changes → **rust-developer**

Cross-crate changes (e.g. adding a new command end-to-end) are coordinated from
this top level; delegate per-crate pieces to the relevant subagent.

## Code navigation: prefer LSP over grep

rust-analyzer is wired up in this repo. For navigating Rust code, **default
to the `LSP` tool** (`documentSymbol`, `workspaceSymbol`, `goToDefinition`,
`findReferences`, `goToImplementation`, `incomingCalls`) rather than `grep`,
`rg`, or `Grep`. LSP returns semantically resolved answers at *current* line
numbers — `findReferences` hits real call sites and skips comment/string
matches; `workspaceSymbol` lists every impl of a trait without false
positives from prose.

`grep` / `Glob` are still the right tool for:

- non-Rust files (TOML, Markdown, JSON, shell)
- free-form text searches (TODO/FIXME, error-message strings, log lines)
- listing files by name or extension
- one-shot literal lookups where the symbol name *is* the search

For everything else — "where is X defined", "who calls Y", "what implements
this trait", "show me the symbol tree of this module" — start with LSP.

**This applies to subagents too.** When delegating to `Explore`, `Plan`, or a
crate-specific subagent, brief them to prefer LSP for code questions. Line
numbers in prior conversations, journals, and stale docs rot quickly; LSP
always returns the current ones.

## Toolchain & Workflow

- Stable Rust, pinned in `rust-toolchain.toml`. Do not introduce nightly features.
- Before considering work "done":
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- New public items get rustdoc. Examples in doc comments are run as tests.
- `unsafe` is disallowed outside explicit justification in a code comment.

### Lint exceptions (workspace-wide)

These are deliberate, set in the root `Cargo.toml` `[workspace.lints.clippy]`
block. Don't undo them without a reason.

- **`clippy::unused_async = "allow"`** — every command handler is
  `pub async fn run(args, ctx) -> Result<()>` by design. They `.await` the
  typed API client once it lands; M0 stubs that don't yet await would
  otherwise trip `pedantic`. Keeping the signature uniform across all
  handlers matters more than catching a stub that hasn't awaited yet.
- **`clippy::module_name_repetitions`, `must_use_candidate`,
  `missing_errors_doc` = `"allow"`** — pragmatic carve-outs from `pedantic`
  that would otherwise produce noise without commensurate value in a CLI.

### Per-item allows currently in tree

- `crates/confluence-cli/src/context.rs` — `#[allow(dead_code)]` on `Context`.
  Its fields (`host`, `output`, `verbose`, `no_color`) are parsed by clap
  in M0 but not yet read by any handler; M1 wires them through. Remove the
  allow as soon as the first handler reads from the struct.

## Error Handling

- Library crates (`confluence-api`, `confluence-auth`) use `thiserror`-derived
  error types. Each crate exposes one top-level `Error` enum, re-exported.
- The binary crate uses `anyhow` (or `color-eyre`) at the boundary, with
  `.context(...)` annotations that read as user-facing breadcrumbs.
- Never `unwrap()` or `expect()` on anything that can fail at runtime, including
  `Mutex` locks under contention. Tests are the only exception.
- Never `panic!()` on bad input. Map to a typed error.

## Async & Logging

- Async via `tokio` (multi-thread runtime in the binary, no runtime in libraries
  — libraries are runtime-agnostic where practical).
- Logging via `tracing`. Use `tracing::instrument` on public async fns where
  it aids debugging. `--verbose` raises the log level; tokens and credentials
  **never** appear in any log, at any level.

## Documentation

- User-facing docs live in `Docs/src/` as mdBook chapters.
- Per-command reference is generated from `clap` definitions — do not write it
  by hand; update the clap structs and let the generator regenerate.
- When adding a feature, add or update the relevant chapter in the same PR.

## Commits & PRs

- Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`,
  `test:`). Scope optional but encouraged: `feat(api): ...`.
- One logical change per commit. PRs squash-merge by default.
- User-visible decisions are captured as ADRs (`Docs/src/adr/`), not as a
  separate changelog. Release notes can be derived from git log + ADR
  history at tag time.

## Architecture Decision Records

Significant decisions are recorded as ADRs under `Docs/src/adr/`, in Michael
Nygard's format, managed with [adr-tools](https://github.com/npryce/adr-tools).
The location is configured in `.adr-dir` at the repository root.

Before changing one of the workspace-shaping decisions (crate boundaries,
scope, lint policy), check whether an existing ADR covers it. ADRs are
immutable once **Accepted** — supersede them with a new ADR rather than
editing the body. New decisions worth recording: pick `adr new "..."`
to scaffold one.

## References

- Project plan & roadmap: discuss with the user before deviating.
- ADRs: `Docs/src/adr/` (rendered in the mdBook under *Architecture*).
- mdBook docs: `Docs/src/`
- Confluence Cloud REST v2: pages, spaces, attachments, comments.
- Confluence Cloud REST v1: search (CQL), users.
