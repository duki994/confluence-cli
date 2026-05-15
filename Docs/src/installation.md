# Installation

> **v0.1 status:** binary distribution lands in M5 (`cargo-dist` cross-platform
> builds, install script, Homebrew tap). Until then, build from source.

## From source

```sh
git clone https://github.com/duki994/confluence-cli.git
cd confluence-cli
cargo install --path crates/confluence-cli
```

This installs the `confluence` binary into `~/.cargo/bin/`. Make sure that
directory is on your `PATH`.

## Verifying

```sh
confluence --help
```

Should print the top-level usage with the noun subcommands (`auth`, `page`,
`space`, `search`, `attachment`, `comment`, `api`).

## Future channels

Planned for M5+:

- Pre-built binaries on every GitHub Release (Linux glibc + musl, macOS
  Intel + Apple Silicon, Windows MSVC).
- `curl … | sh` install script.
- Homebrew tap.
- Possibly Scoop, AUR, Nix flake — depending on demand.
