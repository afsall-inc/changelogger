# Changelogger — Agent Guide

## Agentic Loop

Start every session with:

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

## Data Files

- `changelogger.toml` — project configuration
- `prdoc/` — PR documentation files (`.prdoc`)
- `templates/` — Tera templates for changelog rendering
- `packages/cli/src/schema_user.json` — embedded JSON Schema for `init` command

## Architecture

| Package | Crate | Role |
|---------|-------|------|
| `prdoc` | `changelogger-prdoc` | Core library: types, parse, validate, analyze, generate, changelog |
| `cli` | `changelogger-cli` | Binary entrypoint (`changelogger` command) |

`changelogger-cli` depends on `changelogger-prdoc` via path during development, and via crates.io for published releases.

## Toolchain

- **Rust**: `nightly-2026-02-18` (pinned in `rust-toolchain.toml`)
- **Cargo**: edition 2024, resolver "2"
- **License**: Apache-2.0 OR MIT

## Developer Commands

```bash
mise run ci       # fmt → headers → clippy → test
mise run fmt      # cargo fmt --all
mise run headers  # forehead-cli apply
mise run clippy   # cargo clippy --workspace -- -D warnings
mise run test     # cargo test --workspace
mise run build    # cargo build --workspace

# Single-package
cargo test -p changelogger-prdoc
cargo clippy -p changelogger-prdoc -- -D warnings
```

## Casing

| Item | Convention | Example |
|------|-----------|---------|
| Rust vars | snake_case | `prdoc_path` |
| Files | kebab-case | `changelog.rs` |
| Types | PascalCase | `ChangelogEntry` |

## Testing

- `cargo test --workspace` for unit/integration tests
- 16 tests in `changelogger-prdoc` covering types, config, analyzer, changelog, generator, workspace
- All tests must be hermetic, deterministic, and isolated

## Development Lifecycle

```
1. Create branch → make changes → commit
2. Push → open PR
3. PRDoc CI generates prdoc/pr_N.prdoc from the diff
4. PR merged to main
5. CD detects version bump in Cargo.toml
6. CD generates CHANGELOG.md from prdoc/ directory
7. CD commits CHANGELOG.md + creates vX.Y.Z tag
8. Release workflow: GitHub Release, crates.io, Docker
```

## PRDoc

Structured PR docs at `prdoc/`. Commands:

```bash
# Generate a prdoc from a PR number (requires gh CLI)
cargo run --package changelogger-cli -- prdoc generate --pr 42

# Generate from a git commit range
cargo run --package changelogger-cli -- prdoc generate --from v0.1.0..HEAD

# Validate
cargo run --package changelogger-cli -- prdoc validate prdoc/pr_42.prdoc
cargo run --package changelogger-cli -- prdoc validate prdoc/

# Display as JSON
cargo run --package changelogger-cli -- prdoc show prdoc/pr_42.prdoc
```

## Changelog

```bash
# Generate from prdoc directory
cargo run --package changelogger-cli -- changelog generate --dir prdoc

# Generate from git range
cargo run --package changelogger-cli -- changelog generate --from v0.1.0

# Compute version bumps
cargo run --package changelogger-cli -- changelog bump --current 0.1.0

# Verify all commits have prdocs
cargo run --package changelogger-cli -- changelog verify --from v0.1.0
```

## Publishing

When bumping the version in `Cargo.toml`, the CD workflow handles everything automatically:

1. Push the bump to main
2. CD detects the version change
3. Generates `CHANGELOG.md` from prdoc files
4. Commits it and creates a `vX.Y.Z` tag
5. That tag triggers the Release workflow

For manual publishing:

```bash
# Switch cli to crates.io dependency first
cargo publish -p changelogger-prdoc
cargo publish -p changelogger-cli
# Then restore path dependency for local dev
```

## CI/CD

| Workflow | File | Trigger | Behavior |
|----------|------|---------|----------|
| CI | `.github/workflows/ci.yml` | push/PR to main | fmt → headers → clippy → test → build |
| CD | `.github/workflows/cd.yml` | push to main (version bump) | generate CHANGELOG, tag, publish crates.io + Docker |
| PRDoc | `.github/workflows/prdoc.yml` | PR opened/sync | auto-generate prdoc from diff, validate, commit back |
| Command PRDoc | `.github/workflows/command-prdoc.yml` | workflow_dispatch | manual prdoc generation with bump/audience inputs |
| Release | `.github/workflows/release.yml` | tag push `v*` | GitHub Release + crates.io + Docker |