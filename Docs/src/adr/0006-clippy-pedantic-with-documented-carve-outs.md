# 6. Clippy pedantic with documented carve-outs

Date: 2026-05-16

## Status

Accepted

## Context

Clippy ships several lint groups with very different opinion levels:

- `clippy::correctness` and `clippy::all` (which includes `correctness`,
  `suspicious`, `style`, `complexity`, `perf`) — broadly uncontroversial.
- `clippy::pedantic` — high signal on idiomatic Rust but also high noise
  on items that are stylistic preferences rather than real issues.
- `clippy::nursery`, `clippy::restriction` — experimental or opinionated
  enough that workspace-wide adoption is a poor fit.

We want to land idiomatic Rust early and avoid relitigating lint findings in
every PR, without paying the cost of pedantic's noisier checks. We also have
one project-specific shape that pedantic doesn't like out of the box: every
command handler is `pub async fn run(args, ctx) -> Result<()>`. In M0 the
handlers are stubs that return `not yet implemented` without awaiting anything
— `clippy::unused_async` would fire on all of them, even though they will all
`.await` the HTTP client from M1 onward.

## Decision

Enable `clippy::all` and `clippy::pedantic` at workspace scope in the root
`Cargo.toml`, with these explicit `allow`s:

- `clippy::module_name_repetitions` — common in idiomatic Rust crate layouts.
- `clippy::must_use_candidate` — noisy on a CLI without commensurate value.
- `clippy::missing_errors_doc` — covered by typed `Error` enums in the
  library crates.
- `clippy::unused_async` — required by the uniform handler signature in
  `confluence-cli/src/commands/`.

Each carve-out is annotated inline in `Cargo.toml` with its reason. CI runs:

```
cargo clippy --workspace --all-targets -- -D warnings
```

so any remaining lint blocks merge.

Per-item `#[allow(...)]` is allowed when it carries an inline comment
explaining why. The current tree has one such allow on
`confluence-cli/src/context.rs::Context` (dead-code on fields that M1
handlers will read).

## Consequences

- PR reviewers don't argue about pedantic findings — the lint suite enforces
  the rule.
- Adding a new workspace-wide `allow` without an inline justification is a
  review smell.
- `clippy::unused_async` is on borrowed time — once every handler genuinely
  awaits, this allow should be scoped down (to the `commands` module) or
  removed. Captured here so we don't keep it on by inertia.
- Contributors are insulated from pedantic's churn between Rust releases —
  if a new pedantic lint lands and proves noisy, the carve-out goes in
  `Cargo.toml` with a justification, in this ADR's spirit.
