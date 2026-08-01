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
mise run ci       # fmt → clippy → test
mise run fmt      # cargo fmt --all
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

## PRDoc

Structured PR docs at `prdoc/`. Commands:

```bash
cargo run --package changelogger-cli -- prdoc validate
cargo run --package changelogger-cli -- prdoc show prdoc/pr_1.prdoc
cargo run --package changelogger-cli -- prdoc generate --pr 42
cargo run --package changelogger-cli -- changelog generate --from v0.1.0
```

## Publishing

```bash
# Switch cli to crates.io dependency first
cargo publish -p changelogger-prdoc
cargo publish -p changelogger-cli
# Then restore path dependency for local dev
```

## CI/CD

| Workflow | File | Trigger | Behavior |
|----------|------|---------|----------|
| CI | `.github/workflows/ci.yml` | push/PR to main | fmt → clippy → test → build |
| CD | `.github/workflows/cd.yml` | push to main (version bump) | build + push Docker image to ghcr.io |
| PRDoc | `.github/workflows/prdoc.yml` | PR opened/sync | auto-generate prdoc, validate, commit back |
| Command PRDoc | `.github/workflows/command-prdoc.yml` | workflow_dispatch | manual prdoc generation with bump/audience inputs |
| Release | `.github/workflows/release.yml` | tag push `v*` | GitHub Release + Docker image push to ghcr.io |
