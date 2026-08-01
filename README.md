# changelogger

**changelogger** is a Rust tool for generating structured PR documentation (prdocs) and changelogs. It's designed for CI/CD — every PR gets a machine-readable prdoc file that describes what changed, which crates were affected, and what SemVer bump each crate needs. At release time, prdocs are aggregated into a `CHANGELOG.md`.

Inspired by the [Polkadot SDK](https://github.com/paritytech/polkadot-sdk) prdoc system. Used by [MontRS](https://github.com/afsall-labs/montrs) and [Setheum](https://github.com/setheum/setheum).

## How It Works

```
PR opened → PRDoc workflow generates prdoc/pr_N.prdoc
                ↓
PR merged → CD workflow detects version bump
                ↓
Generates CHANGELOG.md from prdoc/ directory
                ↓
Commits CHANGELOG.md + creates vX.Y.Z tag
                ↓
Release workflow: GitHub Release, crates.io publish, Docker push
```

## Quick Start

### 1. Install

```bash
cargo install changelogger-cli
```

### 2. Init a project

```bash
cd my-rust-project
changelogger prdoc init
```

This creates:
- `changelogger.toml` — project configuration
- `prdoc/schema_user.json` — JSON Schema for validation

### 3. Add CI/CD

Copy these workflows from changelogger's own `.github/workflows/`:

- **`prdoc.yml`** — on PR open/sync, auto-generates prdoc from the diff, validates it, commits it back
- **`cd.yml`** — on push to main, detects version bumps, generates CHANGELOG, creates tags, publishes to crates.io + Docker
- **`release.yml`** — on tag push, creates GitHub Release + publishes

### 4. Make a PR

Create a PR, and the PRDoc workflow will:
1. Fetch the diff, title, and body via `gh`
2. Analyze the diff to determine which crates changed and what bump levels
3. Generate `prdoc/pr_N.prdoc` 
4. Validate it
5. Commit it to the PR branch

### 5. Release

Merge to main. When the version in `Cargo.toml` changes, the CD workflow:
1. Generates `CHANGELOG.md` from all prdoc files
2. Commits it
3. Creates a `vX.Y.Z` tag and pushes it
4. Publishes to crates.io and ghcr.io

## CLI

```
changelogger prdoc init                          # Scaffold configuration
changelogger prdoc generate --pr 42              # From GitHub PR (gh CLI)
changelogger prdoc generate --from v1.0.0..HEAD  # From git commit range
changelogger prdoc generate --diff patch.diff    # From a diff file
changelogger prdoc validate                      # Validate all prdoc files
changelogger prdoc show prdoc/pr_42.prdoc        # Display as JSON
changelogger changelog generate --from v1.0.0    # Generate CHANGELOG.md from git range
changelogger changelog generate --dir prdoc      # Generate CHANGELOG.md from prdoc directory
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

## PRDoc Format

A prdoc file is a YAML file in `prdoc/pr_N.prdoc`:

```yaml
---
title: "Add XCM send support to ink_env"
author: @username
pr: 42
doc:
  - audience: Developer
    description: |
      Adds xcm_send for contracts to submit XCM messages.
crates:
  - name: ink_env
    bump: minor
---
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `title` | yes | Short description of the change |
| `author` | no | GitHub handle |
| `pr` | no | PR number |
| `doc` | yes | Array of audience-specific descriptions |
| `crates` | yes | Array of affected crates with bump levels |
| `migrations` | no | Database and runtime migrations |
| `host_functions` | no | Host function changes |

### Bump Levels

| Bump | When to use |
|------|-------------|
| `major` | Breaking public API changes |
| `minor` | New public API additions |
| `patch` | Bug fixes, internal changes |
| `none` | No observable change (docs, CI, comments) |

### Audiences

| Audience | Who they are |
|----------|-------------|
| `Developer` | People consuming the library or writing code against it |
| `User` | End users of the tool |
| `Operator` | CI/CD pipeline maintainers |

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

## CI/CD

### PRDoc Workflow (auto-generate + validate on PR)

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

### CD Workflow (auto-changelog + publish on version bump)

```yaml
# .github/workflows/cd.yml
name: CD
on:
  push:
    branches: [main]
jobs:
  detect:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
      changed: ${{ steps.version.outputs.changed }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 2
      - id: version
        run: |
          VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"
          if git show HEAD~1:Cargo.toml | grep -q "^version = \"$VERSION\""; then
            echo "changed=false" >> "$GITHUB_OUTPUT"
          else
            echo "changed=true" >> "$GITHUB_OUTPUT"
          fi

  changelog:
    needs: detect
    if: needs.detect.outputs.changed == 'true'
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@v1
        with: { toolchain: stable }
      - uses: Swatinem/rust-cache@v2
      - run: cargo install changelogger-cli
      - run: changelogger changelog generate --dir prdoc --output CHANGELOG.md
      - run: |
          VERSION=${{ needs.detect.outputs.version }}
          git config user.name "bot"
          git config user.email "bot@github.com"
          git add CHANGELOG.md
          git commit -m "changelog: auto-generate for v${VERSION}"
          git tag "v${VERSION}"
          git push origin "v${VERSION}"
```

## Crates

| Crate | Description |
|-------|-------------|
| [changelogger-prdoc](https://crates.io/crates/changelogger-prdoc) | Core library: types, parse, validate, analyze, changelog |
| [changelogger-cli](https://crates.io/crates/changelogger-cli) | CLI binary (`cargo install changelogger-cli`) |

## Docker

```bash
docker pull ghcr.io/afsall-inc/changelogger:latest
docker run ghcr.io/afsall-inc/changelogger --help
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT), at your option.