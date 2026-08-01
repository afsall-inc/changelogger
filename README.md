# changelogger

**changelogger** is a Rust tool for generating structured PR documentation (prdocs) and changelogs. It's designed for CI/CD — every PR gets a machine-readable prdoc file that describes what changed, which crates were affected, and what SemVer bump each crate needs. At release time, prdocs are aggregated into a `CHANGELOG.md`.

Inspired by the [Polkadot SDK](https://github.com/paritytech/polkadot-sdk) prdoc system. Used by [MontRS](https://github.com/afsall-inc/montrs) and [Setheum](https://github.com/setheum/setheum).

## Why PRDoc?

PRs are the best source of truth for what changed. But PR descriptions are free-form and hard to automate. prdoc files are structured YAML files that live alongside your code:

```yaml
title: "Add XCM send support to ink_env"
doc:
  - audience: Developer
    description: |
      Adds xcm_send for contracts to submit XCM messages.
crates:
  - name: ink_env
    bump: minor
```

This enables:
- **Automated changelogs** — no manual release notes
- **SemVer tracking** — know exactly which crates need bumping
- **Audience-targeted docs** — different descriptions for devs, users, operators
- **CI enforcement** — every PR must document its changes

## Quick Start

```bash
# Install
cargo install changelogger-cli

# Init a project
cd my-rust-project
changelogger prdoc init
# Creates: changelogger.toml, prdoc/schema_user.json

# Generate a prdoc for a PR (requires gh CLI)
changelogger prdoc generate --pr 42

# Validate
changelogger prdoc validate prdoc/pr_42.prdoc

# At release time, generate changelog
changelogger changelog generate --from v0.1.0
```

## CLI

```
changelogger prdoc init                          # Scaffold configuration
changelogger prdoc generate --pr 42              # From GitHub PR (gh CLI)
changelogger prdoc generate --from v1.0.0..HEAD  # From git commit range
changelogger prdoc generate --diff patch.diff    # From a diff file
changelogger prdoc validate                      # Validate all prdoc files
changelogger prdoc show prdoc/pr_42.prdoc        # Display as JSON
changelogger changelog generate --from v1.0.0    # Generate CHANGELOG.md
changelogger changelog bump --current 0.1.0      # Compute next versions
changelogger changelog verify --from v1.0.0      # Check all commits have prdocs
```

## Configuration

`changelogger.toml` in your project root:

```toml
version = 1
schema = "prdoc/schema_user.json"
output_dir = "prdoc"
prdoc_folders = ["prdoc"]
template = "prdoc/.template.prdoc"

[audiences]
developer = "Developer"
user = "User"
operator = "Operator"
```

### Audiences

Each doc section targets an audience. Defaults: `Developer`, `User`, `Operator`.
Override them in `changelogger.toml`:

```toml
[audiences]
developer = "Runtime Dev"
user = "App Dev"
operator = "Node Operator"
```

## CI/CD

### Auto-generate + validate on PR

```yaml
# .github/workflows/prdoc.yml
name: PRDoc
on:
  pull_request:
    types: [opened, synchronize, reopened]
permissions:
  contents: write
jobs:
  prdoc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
          ref: ${{ github.head_ref }}
      - uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install changelogger-cli
      - name: Generate PRDoc
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          PR_NUM=${{ github.event.pull_request.number }}
          if [ ! -f "prdoc/pr_${PR_NUM}.prdoc" ]; then
            changelogger prdoc generate --pr "$PR_NUM" --force
          fi
      - name: Validate
        run: changelogger prdoc validate prdoc/pr_${{ github.event.pull_request.number }}.prdoc
      - uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: "Add prdoc for PR #${{ github.event.pull_request.number }}"
          file_pattern: "prdoc/*.prdoc"
```

## Changelog Format

Generated changelogs follow [Keep a Changelog](https://keepachangelog.com/) with entries grouped by category:

```
# Changelog

## [Unreleased]

### Added

- Add XCM send support to ink_env [ink_env(patch)] (#42)

### Fixed

- Fix null pointer in storage [ink_storage(patch)] (#41)
```

Categories are inferred from bump levels: `major` → Removed, `minor` → Added, `patch` → Fixed.

## Crates

| Crate | Description |
|-------|-------------|
| [changelogger-prdoc](https://crates.io/crates/changelogger-prdoc) | Core library: types, parse, validate, analyze, changelog |
| [changelogger-cli](https://crates.io/crates/changelogger-cli) | CLI binary (`cargo install changelogger-cli`) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT), at your option.

## Docker

Pre-built images are available on GitHub Container Registry:

```bash
docker pull ghcr.io/afsall-inc/changelogger:latest
docker run ghcr.io/afsall-inc/changelogger --help
```

Images are published automatically on version bumps (via `cd.yml`) and on tag push `v*` (via `release.yml`).
