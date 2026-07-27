# changelogger-cli

CLI tool for changelogger — generate PR documentation and changelogs for Rust projects.

## Installation

```bash
cargo install changelogger-cli
```

## Usage

```
changelogger prdoc init                          # Scaffold changelogger.toml + prdoc/ directory
changelogger prdoc generate --pr 42              # Generate prdoc from GitHub PR via gh CLI
changelogger prdoc generate --from v1.0.0..HEAD  # Generate prdoc from git commit range
changelogger prdoc generate --diff patch.diff    # Generate prdoc from a diff file
changelogger prdoc validate                      # Validate all prdoc/*.prdoc files
changelogger prdoc show prdoc/pr_42.prdoc        # Display a prdoc as JSON
changelogger changelog generate --from v1.0.0    # Generate CHANGELOG.md from prdocs
changelogger changelog bump --current 0.1.0      # Compute next version bumps
changelogger changelog verify --from v1.0.0      # Check all commits have prdocs
```

## CI/CD

Add to `.github/workflows/prdoc.yml`:

```yaml
name: PRDoc
on: pull_request
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: stable
      - run: cargo install changelogger-cli
      - run: changelogger prdoc generate --pr ${{ github.event.pull_request.number }} --force
      - run: changelogger prdoc validate
```

## License

Licensed under [Apache-2.0 OR MIT](https://github.com/afsall-inc/changelogger) at your option.