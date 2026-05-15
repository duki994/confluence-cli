# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the project is pre-1.0, breaking changes between minor versions are
expected.

## [Unreleased]

### Added

- M0 skeleton: Cargo workspace with three crates (`confluence-api`,
  `confluence-auth`, `confluence-cli`).
- `clap`-driven command surface for `auth`, `page`, `space`, `search`,
  `attachment`, `comment`, and `api` (all return `not yet implemented`).
- mdBook documentation scaffold under `Docs/`.
- GitHub Actions workflows: `ci.yml` (fmt + clippy + test on Linux/macOS/Windows)
  and `docs.yml` (mdBook build + GitHub Pages deploy).
- `CLAUDE.md` and three `.claude/agents/` subagent definitions covering the
  per-crate boundaries.
