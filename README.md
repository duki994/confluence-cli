# confluence-cli

Command-line tool for Atlassian Confluence, written in Rust, influenced by [`gh`](https://cli.github.com/) CLI. Binary name: `confluence`.

> ⚠️ **Status: M0 skeleton.** The command surface is in place but every
> handler returns `not yet implemented`. Real functionality starts landing
> in M1 (auth + generic API). See
> [`Docs/src/intro.md`](./Docs/src/intro.md) for the milestone roadmap and
> [`Docs/src/adr/`](./Docs/src/adr/) for the architecture decision records.

## Scope (v0.1)

Aggressively narrow on purpose:

- **Atlassian Cloud only.** Server / Data Center support is on the roadmap.
- **API-token auth only.** OAuth and PAT come later.
- **Confluence storage format only.** Markdown ↔ storage conversion is
  on the roadmap.

## Build & run

Requires a stable Rust toolchain (the version is pinned in
`rust-toolchain.toml` and `rustup` will install it automatically on first
build).

```sh
cargo build --workspace
cargo run -p confluence-cli -- --help
```

## Workspace layout

```
crates/
  confluence-api/   typed Confluence REST client
  confluence-auth/  credential storage + host registry
  confluence-cli/   the `confluence` binary (clap, output formatting)
Docs/               mdBook source, deployed to GitHub Pages
```

## Contributing

See [`CLAUDE.md`](./CLAUDE.md) for the project conventions (toolchain,
error handling, commits) and [`.claude/agents/`](./.claude/agents) for the
per-crate boundaries that pair with each subagent.

Before opening a PR:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## License

MIT — see [`LICENSE`](./LICENSE).
