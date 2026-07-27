# changelogger-prdoc

Structured PR documentation, auto-generation, changelog, and SemVer bumping for Rust projects. Usable standalone — no framework dependency.

## Features

- **Parse & Validate** — YAML frontmatter PR docs with JSON Schema validation
- **Auto-generate** — from GitHub PR number, git commit range, or raw diff
- **Diff analysis** — commit classification, breaking change detection, crate mapping
- **Changelog** — Keep a Changelog format with category grouping, Tera templates, SemVer bumps
- **Workspace-aware** — uses `cargo_metadata` to map file paths to crates in any workspace structure

## Library Usage

```rust
use changelogger_prdoc::{parse_prdoc, validate_prdoc, PrDoc};

let content = std::fs::read_to_string("prdoc/pr_1.prdoc").unwrap();
let prdoc: PrDoc = parse_prdoc(&content).unwrap();
let issues = validate_prdoc(&prdoc);
```

```rust
use changelogger_prdoc::{Changelog, load_prdoc};

let prdoc = load_prdoc(&std::path::Path::new("prdoc/pr_1.prdoc")).unwrap();
let mut changelog = Changelog::new();
changelog.add_prdoc(&prdoc);
std::fs::write("CHANGELOG.md", changelog.render()).unwrap();
```

## Configuration

Create `changelogger.toml` in your project root:

```toml
version = 1
schema = "prdoc/schema_user.json"
output_dir = "prdoc"
prdoc_folders = ["prdoc"]

[audiences]
developer = "Developer"
user = "User"
operator = "Operator"
```

## License

MIT