# Changelogger — Agent Guide

## Agentic Loop

Start every session with:

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

## Data Files

- `.prdoc.toml` — project configuration
- `prdoc/` — PR documentation files
- `templates/` — Tera templates for changelog rendering

## Architecture

| Package | Role |
|---------|------|
| `prdoc` | Core library: types, parse, validate, analyze, generate, changelog |
| `cli` | Binary entrypoint (`changelogger` command) |

## Toolchain

- **Rust**: `nightly-2026-02-18` (pinned in `rust-toolchain.toml`)
- **Cargo**: edition 2024, resolver "2"

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
- All tests must be hermetic, deterministic, and isolated

## PRDoc

Structured PR docs at `prdoc/`. Commands:

```bash
cargo run -- prdoc validate
cargo run -- prdoc show prdoc/pr_1.prdoc
cargo run -- prdoc generate --pr 42
cargo run -- changelog generate --from v0.1.0
```