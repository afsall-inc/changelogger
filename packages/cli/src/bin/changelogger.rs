// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
// This file is part of changelogger.
//
// Copyright (C) 2026-Present Afsall Inc.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Alternatively, this file is available under the MIT License:
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "changelogger",
    version,
    about = "Auto-generate CHANGELOGs and prdocs"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// PR documentation operations
    Prdoc {
        #[command(subcommand)]
        cmd: PrdocCmd,
    },
    /// Changelog operations
    Changelog {
        #[command(subcommand)]
        cmd: ChangelogCmd,
    },
    /// CI/CD workflow scaffolding
    Ci {
        #[command(subcommand)]
        cmd: CiCmd,
    },
    /// Publish crates to crates.io
    Publish {
        /// Crate to publish (default: all workspace crates)
        #[arg(long)]
        crate_name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum PrdocCmd {
    /// Initialize prdoc directory and config
    Init,
    /// Display a prdoc file as JSON
    Show {
        #[arg(default_value = "prdoc/pr_1.prdoc")]
        path: String,
    },
    /// Validate prdoc files
    Validate {
        /// Path to a single prdoc file, or directory of prdoc files
        #[arg(default_value = "prdoc")]
        path: String,
        /// Branch name for backport validation
        #[arg(long)]
        branch: Option<String>,
    },
    /// Auto-generate a prdoc skeleton
    Generate {
        /// PR number (uses gh CLI)
        #[arg(short, long)]
        pr: Option<u64>,
        /// Git commit range (e.g., v1.0.0..HEAD)
        #[arg(long)]
        from: Option<String>,
        /// Path to a diff file
        #[arg(long)]
        diff: Option<String>,
        /// Default bump level
        #[arg(short, long, default_value = "minor")]
        bump: String,
        /// Default audience
        #[arg(short, long, default_value = "developer")]
        audience: String,
        /// Overwrite existing files
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ChangelogCmd {
    /// Generate CHANGELOG.md from prdoc files
    Generate {
        /// Git range to scan for prdocs (e.g., v1.0.0..HEAD)
        #[arg(short, long)]
        from: Option<String>,
        /// Output file
        #[arg(short, long, default_value = "CHANGELOG.md")]
        output: String,
        /// Directory with prdoc files instead of git range
        #[arg(long)]
        dir: Option<String>,
        /// Release version (e.g. 0.2.0). Writes a versioned section into the
        /// existing changelog instead of overwriting with only [Unreleased]
        #[arg(long)]
        release: Option<String>,
    },
    /// Compute version bumps from prdocs
    Bump {
        /// Current version
        #[arg(short, long, default_value = "0.1.0")]
        current: String,
        /// Git range
        #[arg(long)]
        from: Option<String>,
        /// Dry run (just print, don't modify)
        #[arg(long)]
        dry_run: bool,
    },
    /// Verify all commits in a range have prdocs
    Verify {
        /// Git range
        #[arg(long)]
        from: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum CiCmd {
    /// Scaffold CI/CD workflows (prdoc.yml, cd.yml, release.yml, ci.yml)
    Init,
}

fn main() {
    let cli = Cli::parse();
    let result = run_command(cli);
    match result {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

fn run_command(cli: Cli) -> Result<String, String> {
    match cli.command {
        Commands::Prdoc { cmd } => run_prdoc(cmd),
        Commands::Changelog { cmd } => run_changelog(cmd),
        Commands::Ci { cmd } => run_ci(cmd),
        Commands::Publish { crate_name } => cmd_publish(crate_name.as_deref()),
    }
}

fn run_prdoc(cmd: PrdocCmd) -> Result<String, String> {
    match cmd {
        PrdocCmd::Init => cmd_init(),
        PrdocCmd::Show { path } => cmd_show(&path),
        PrdocCmd::Validate { path, branch } => {
            cmd_validate(&path, branch.as_deref())
        }
        PrdocCmd::Generate {
            pr,
            from,
            diff,
            bump,
            audience,
            force,
        } => cmd_generate(
            pr,
            from.as_deref(),
            diff.as_deref(),
            &bump,
            &audience,
            force,
        ),
    }
}

fn run_changelog(cmd: ChangelogCmd) -> Result<String, String> {
    match cmd {
        ChangelogCmd::Generate {
            from,
            output,
            dir,
            release,
        } => cmd_changelog_generate(
            from.as_deref(),
            &output,
            dir.as_deref(),
            release.as_deref(),
        ),
        ChangelogCmd::Bump {
            current,
            from,
            dry_run,
        } => cmd_changelog_bump(&current, from.as_deref(), dry_run),
        ChangelogCmd::Verify { from } => cmd_changelog_verify(from.as_deref()),
    }
}

fn run_ci(cmd: CiCmd) -> Result<String, String> {
    match cmd {
        CiCmd::Init => cmd_ci_init(),
    }
}

fn cmd_ci_init() -> Result<String, String> {
    let root = std::env::current_dir().map_err(|e| format!("{e}"))?;
    let workflows_dir = root.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows_dir).map_err(|e| format!("{e}"))?;

    let workflows = [
        ("ci.yml", CI_WORKFLOW),
        ("prdoc.yml", PRDOC_WORKFLOW),
        ("cd.yml", CD_WORKFLOW),
        ("release.yml", RELEASE_WORKFLOW),
    ];

    let mut written = Vec::new();
    for (name, content) in &workflows {
        let path = workflows_dir.join(name);
        if path.exists() {
            written.push(format!("{name} (already exists, skipped)"));
        } else {
            std::fs::write(&path, content).map_err(|e| format!("{e}"))?;
            written.push(name.to_string());
        }
    }

    Ok(format!(
        "Scaffolded {} workflow(s) in .github/workflows/:\n  - {}",
        written.len(),
        written.join("\n  - "),
    ))
}

fn cmd_init() -> Result<String, String> {
    let root = std::env::current_dir().map_err(|e| format!("{e}"))?;

    let prdoc_dir = root.join("prdoc");
    std::fs::create_dir_all(&prdoc_dir).map_err(|e| format!("{e}"))?;

    let config_path = root.join("changelogger.toml");
    if !config_path.exists() {
        let config = r#"version = 1
schema = "prdoc/schema_user.json"
output_dir = "prdoc"
prdoc_folders = ["prdoc"]
template = "prdoc/.template.prdoc"

[audiences]
developer = "Developer"
user = "User"
operator = "Operator"
"#;
        std::fs::write(&config_path, config).map_err(|e| format!("{e}"))?;
    }

    let schema_path = prdoc_dir.join("schema_user.json");
    if !schema_path.exists() {
        std::fs::write(&schema_path, include_str!("../schema_user.json"))
            .map_err(|e| format!("{e}"))?;
    }

    Ok(format!("Initialized prdoc in {}", prdoc_dir.display()))
}

fn cmd_show(path: &str) -> Result<String, String> {
    let prdoc_path = PathBuf::from(path);
    if !prdoc_path.exists() {
        return Err(format!("File not found: {path}"));
    }
    let prdoc = changelogger_prdoc::load_prdoc(&prdoc_path)
        .map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&prdoc).map_err(|e| e.to_string())
}

fn cmd_validate(path: &str, branch: Option<&str>) -> Result<String, String> {
    let path = PathBuf::from(path);

    if path.is_dir() {
        let mut all_issues = Vec::new();
        let mut count = 0usize;
        let entries = changelogger_prdoc::walk_dir(&path).unwrap_or_default();
        for entry in entries {
            if entry.extension().and_then(|e| e.to_str()) != Some("prdoc") {
                continue;
            }
            count += 1;
            let prdoc = match changelogger_prdoc::load_prdoc(&entry) {
                Ok(p) => p,
                Err(e) => {
                    all_issues.push(format!("{}: {}", entry.display(), e));
                    continue;
                }
            };
            let issues = if let Some(branch_name) = branch {
                changelogger_prdoc::validate_prdoc_for_branch(
                    &prdoc,
                    branch_name,
                )
            } else {
                changelogger_prdoc::validate_prdoc(&prdoc)
            };
            if !issues.is_empty() {
                all_issues.push(format!(
                    "{}:\n{}",
                    entry.display(),
                    issues
                        .iter()
                        .map(|i| format!("  - {i}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        }
        if all_issues.is_empty() {
            Ok(format!("Validated {count} prdoc file(s)."))
        } else {
            Err(all_issues.join("\n"))
        }
    } else {
        if !path.exists() {
            if std::env::var("CI").is_ok() {
                return Err(format!(
                    "prdoc not found at {}. PRs require prdoc.",
                    path.display()
                ));
            }
            return Ok("No prdoc found. Validation skipped.".to_string());
        }
        let prdoc =
            changelogger_prdoc::load_prdoc(&path).map_err(|e| e.to_string())?;
        let issues = if let Some(branch_name) = branch {
            changelogger_prdoc::validate_prdoc_for_branch(&prdoc, branch_name)
        } else {
            changelogger_prdoc::validate_prdoc(&prdoc)
        };
        if issues.is_empty() {
            Ok("prdoc is valid.".to_string())
        } else {
            let mut out = "Issues:\n".to_string();
            for issue in issues {
                out.push_str(&format!("  - {issue}\n"));
            }
            Err(out)
        }
    }
}

fn cmd_generate(
    pr: Option<u64>,
    from: Option<&str>,
    diff: Option<&str>,
    bump: &str,
    audience: &str,
    force: bool,
) -> Result<String, String> {
    let bump_level = changelogger_prdoc::BumpLevel::from_str_lossy(bump);
    let audience_val = changelogger_prdoc::Audience::from_str_lossy(audience);

    if let Some(pr_number) = pr {
        let opts = changelogger_prdoc::GenerateOptions {
            pr_number,
            bump: bump_level,
            audience: audience_val,
            force,
            workspace: None,
        };

        let prdoc = changelogger_prdoc::generator::generate_prdoc(&opts)
            .map_err(|e| e.to_string())?;

        let output_path =
            changelogger_prdoc::generator::default_output_path(pr_number);
        let path = PathBuf::from(&output_path);

        if path.exists() && !force {
            return Err(format!(
                "{output_path} exists. Use --force to overwrite."
            ));
        }

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let rendered = changelogger_prdoc::generator::render_prdoc(&prdoc);
        std::fs::write(&path, &rendered).map_err(|e| e.to_string())?;

        Ok(format!(
            "Generated {output_path} ({} crate(s)). Edit the `...` \
             placeholders.",
            prdoc.crates.len(),
        ))
    } else if let Some(range) = from {
        let diff_content =
            changelogger_prdoc::analyzer::get_diff_for_range(range)
                .ok_or_else(|| format!("No diff found for range {range}"))?;
        let prdoc = changelogger_prdoc::generator::generate_prdoc_from_diff(
            &diff_content,
            &format!("Changes from {range}"),
            "@changelogger",
            0,
        );
        let rendered = changelogger_prdoc::generator::render_prdoc(&prdoc);
        let output_path = "prdoc/generated.prdoc";
        let path = PathBuf::from(output_path);

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        std::fs::write(&path, &rendered).map_err(|e| e.to_string())?;
        Ok(format!("Generated {output_path} from range {range}"))
    } else if let Some(diff_path) = diff {
        let diff_content = std::fs::read_to_string(diff_path)
            .map_err(|e| format!("Failed to read diff: {e}"))?;
        let prdoc = changelogger_prdoc::generator::generate_prdoc_from_diff(
            &diff_content,
            "Changes from diff",
            "@changelogger",
            0,
        );
        let rendered = changelogger_prdoc::generator::render_prdoc(&prdoc);
        let output_path = "prdoc/generated.prdoc";
        let path = PathBuf::from(output_path);

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        std::fs::write(&path, &rendered).map_err(|e| e.to_string())?;
        Ok(format!("Generated {output_path} from diff {diff_path}"))
    } else {
        Err("One of --pr, --from, or --diff is required".to_string())
    }
}

fn cmd_changelog_generate(
    from: Option<&str>,
    output: &str,
    dir: Option<&str>,
    release: Option<&str>,
) -> Result<String, String> {
    let prdocs = if let Some(dir_path) = dir {
        let path = PathBuf::from(dir_path);
        if !path.exists() {
            return Err(format!("Directory not found: {dir_path}"));
        }
        changelogger_prdoc::load_prdocs_from_dir_recursive(&path)
    } else if let Some(range) = from {
        changelogger_prdoc::collect_prdocs_from_git(range)
    } else {
        let root =
            changelogger_prdoc::find_project_root().ok_or_else(|| {
                "No project root found. Use --from or --dir.".to_string()
            })?;
        let config = changelogger_prdoc::load_config(&root);
        let prdoc_dir = root.join(&config.output_dir);
        if prdoc_dir.exists() {
            changelogger_prdoc::load_prdocs_from_dir_recursive(&prdoc_dir)
        } else {
            return Err(
                "No prdoc directory found. Use --from or --dir.".to_string()
            );
        }
    };

    let mut changelog = changelogger_prdoc::Changelog::new();
    for p in &prdocs {
        changelog.add_prdoc(p);
    }

    if let Some(version) = release {
        let release_section = changelog.render_release(version);
        let path = std::path::Path::new(output);

        let existing = if path.exists() {
            let content =
                std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            // Strip the header lines (everything before ## [Unreleased])
            // so we can keep the header intact and insert the release
            content
        } else {
            String::new()
        };

        let rendered = if existing.is_empty() {
            format!(
                "# Changelog\n\n\
                 All notable changes to this project will be documented in this \
                 file.\n\n\
                 The format is based on [Keep a Changelog]\
                 (https://keepachangelog.com/),\n\
                 and this project adheres to [Semantic Versioning]\
                 (https://semver.org/).\n\n\
                 {release_section}"
            )
        } else {
            // Insert the release section right after the header, before
            // [Unreleased]
            let unreleased_marker = "## [Unreleased]";
            if let Some(pos) = existing.find(unreleased_marker) {
                let before = &existing[..pos];
                let after = &existing[pos..];
                format!("{before}{release_section}\n{after}")
            } else {
                format!("{existing}\n{release_section}")
            }
        };

        std::fs::write(output, &rendered).map_err(|e| e.to_string())?;
    } else {
        let rendered = changelog.render();
        std::fs::write(output, &rendered).map_err(|e| e.to_string())?;
    }

    let source = dir
        .map(|d| d.to_string())
        .or_else(|| from.map(|f| f.to_string()));
    let source_str = source.unwrap_or_else(|| "prdoc/".to_string());
    Ok(format!(
        "Generated {output} with {} entr(ies) from '{source_str}'",
        prdocs.len(),
    ))
}

fn cmd_changelog_bump(
    current: &str,
    from: Option<&str>,
    dry_run: bool,
) -> Result<String, String> {
    let range_default = format!("v{current}..HEAD");
    let range = from.unwrap_or(&range_default);
    let prdocs = changelogger_prdoc::collect_prdocs_from_git(range);
    let bumps = changelogger_prdoc::determine_next_version(current, &prdocs);

    if bumps.is_empty() {
        Ok("No version bumps needed.".to_string())
    } else {
        let mut out = format!("Bumps from {current}:\n");
        for (c, v) in &bumps {
            out.push_str(&format!(
                "  {c} -> {v}{}\n",
                if dry_run { " (dry-run)" } else { "" }
            ));
        }
        Ok(out)
    }
}

fn cmd_changelog_verify(from: Option<&str>) -> Result<String, String> {
    let version = "0.1.0";
    let range_default = format!("v{version}..HEAD");
    let range = from.unwrap_or(&range_default);
    let prdocs = changelogger_prdoc::collect_prdocs_from_git(range);

    let output = std::process::Command::new("git")
        .args(["log", "--oneline", range])
        .output();

    let log_str = output
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let total = log_str.lines().count();
    let missing = total.saturating_sub(prdocs.len());

    if missing == 0 {
        Ok(format!("All {total} commit(s) have prdocs."))
    } else {
        Ok(format!(
            "{missing} commit(s) missing prdocs ({}/{total} found).",
            prdocs.len(),
        ))
    }
}

fn cmd_publish(crate_name: Option<&str>) -> Result<String, String> {
    let root = std::env::current_dir().map_err(|e| format!("{e}"))?;

    let ws = changelogger_prdoc::WorkspaceInfo::load_from_root(&root)
        .map_err(|e| format!("{e}"))?;

    let version = ws.version();
    let major_minor = version
        .rsplitn(2, '.')
        .last()
        .unwrap_or(&version)
        .to_string();

    let publishable: Vec<&changelogger_prdoc::WorkspaceCrate> =
        ws.crates.iter().filter(|c| c.publish).collect();

    let crates: Vec<&changelogger_prdoc::WorkspaceCrate> = if let Some(name) =
        crate_name
    {
        let c =
            publishable.iter().find(|c| c.name == name).ok_or_else(|| {
                format!(
                    "crate '{name}' not found in workspace or not publishable"
                )
            })?;
        vec![c]
    } else {
        let mut sorted = publishable.clone();
        sorted.sort_by_key(|c| workspace_dep_count(&c.manifest_path, &ws));
        sorted
    };

    let mut patched_files: Vec<(PathBuf, String)> = Vec::new();

    for crate_info in &crates {
        println!("Publishing {} v{}...", crate_info.name, version);

        let manifest = &crate_info.manifest_path;
        let orig =
            std::fs::read_to_string(manifest).map_err(|e| format!("{e}"))?;
        let mut patched = orig.clone();

        for ws_crate in &ws.crates {
            if ws_crate.name == crate_info.name {
                continue;
            }
            // Swap `name = { path = "../dir" }` → `name = "major.minor"`
            let path_dep = format!("{} = {{ path = \"", ws_crate.name);
            let ver_dep = format!("{} = \"{}\"", ws_crate.name, major_minor);
            if patched.contains(&path_dep) {
                // Find the start and end of the path dep line, replace it
                let mut result = String::new();
                for line in patched.lines() {
                    if line.trim().starts_with(&path_dep) {
                        result.push_str(&ver_dep);
                        result.push('\n');
                    } else {
                        result.push_str(line);
                        result.push('\n');
                    }
                }
                patched = result;
                break;
            }
        }

        if patched != orig {
            std::fs::write(manifest, &patched).map_err(|e| format!("{e}"))?;
            patched_files.push((manifest.clone(), orig));
        }

        let output = std::process::Command::new("cargo")
            .args(["publish", "-p", &crate_info.name, "--allow-dirty"])
            .output()
            .map_err(|e| format!("failed to run cargo publish: {e}"))?;

        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            println!("Published {} v{}", crate_info.name, version);
        } else if stderr.contains("already exists")
            || stderr.contains("already uploaded")
        {
            println!(
                "{} v{} already published — skipping",
                crate_info.name, version
            );
        } else {
            for (path, content) in &patched_files {
                let _ = std::fs::write(path, content);
            }
            return Err(format!(
                "publish failed for {}:\n{}",
                crate_info.name, stderr
            ));
        }
    }

    for (path, content) in &patched_files {
        std::fs::write(path, content).map_err(|e| format!("{e}"))?;
    }

    Ok("All crates published successfully.".to_string())
}

fn workspace_dep_count(
    manifest_path: &PathBuf,
    ws: &changelogger_prdoc::WorkspaceInfo,
) -> usize {
    let content = match std::fs::read_to_string(manifest_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    ws.crates
        .iter()
        .filter(|c| content.contains(&format!("{} = {{ path = ", c.name)))
        .count()
}

const CI_WORKFLOW: &str = r#"name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings
      - name: Run tests
        run: cargo test --workspace
      - name: Build
        run: cargo build --workspace
"#;

const PRDOC_WORKFLOW: &str = r#"name: PRDoc
on:
  pull_request:
    types: [opened, synchronize, reopened]

permissions:
  contents: write
  pull-requests: write

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
        ref: ${{ github.head_ref }}
    - name: Install Rust
      uses: dtolnay/rust-toolchain@v1
      with:
        toolchain: stable
    - uses: Swatinem/rust-cache@v2
    - name: Install changelogger
      run: cargo install changelogger-cli
    - name: Generate PRDoc
      id: generate
      env:
        GH_TOKEN: ${{ github.token }}
      run: |
        PR_NUM=${{ github.event.pull_request.number }}
        PRDOC="prdoc/pr_${PR_NUM}.prdoc"
        if [ -f "$PRDOC" ]; then
          echo "PRDoc already exists — validating only"
          echo "generated=false" >> "$GITHUB_OUTPUT"
        else
          changelogger prdoc generate --pr "$PR_NUM" --force
          echo "generated=true" >> "$GITHUB_OUTPUT"
        fi
    - name: Validate and auto-fix PRDoc
      env:
        CI: "true"
      run: |
        PRDOC="prdoc/pr_${{ github.event.pull_request.number }}.prdoc"
        changelogger prdoc validate --fix "$PRDOC"
    - name: Validate Backport Rules
      if: startsWith(github.base_ref, 'stable') || startsWith(github.base_ref, 'release')
      run: |
        PRDOC="prdoc/pr_${{ github.event.pull_request.number }}.prdoc"
        changelogger prdoc validate --fix --branch "${{ github.base_ref }}" "$PRDOC"
    - name: Commit PRDoc
      if: steps.generate.outputs.generated == 'true'
      uses: stefanzweifel/git-auto-commit-action@v5
      with:
        commit_message: "Add prdoc for PR #${{ github.event.pull_request.number }} (auto-generated)"
        branch: ${{ github.head_ref }}
        file_pattern: "prdoc/*.prdoc"
"#;

const CD_WORKFLOW: &str = r#"name: CD
on:
  push:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

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
      - name: Detect version bump
        id: version
        run: |
          VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"
          if git show HEAD~1:Cargo.toml 2>/dev/null | grep -q "^version = \"$VERSION\""; then
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
      - name: Install Rust
        uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@v2
      - name: Install changelogger
        run: cargo install changelogger-cli
      - name: Generate CHANGELOG
        run: changelogger changelog generate --dir prdoc --output CHANGELOG.md --release "${{ needs.detect.outputs.version }}"
      - name: Commit and tag
        run: |
          VERSION=${{ needs.detect.outputs.version }}
          git config user.name "changelogger-bot"
          git config user.email "bot@github.com"
          git add CHANGELOG.md
          git commit -m "changelog: auto-generate for v${VERSION}"
          git tag "v${VERSION}"
          git push origin "v${VERSION}"

  crates-io:
    needs: detect
    if: needs.detect.outputs.changed == 'true'
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@v2
      - name: Install changelogger
        run: cargo install changelogger-cli
      - name: Publish to crates.io
        run: changelogger publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

  container:
    needs: detect
    if: needs.detect.outputs.changed == 'true'
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=semver,pattern={{version}},value=v${{ needs.detect.outputs.version }}
            type=semver,pattern={{major}}.{{minor}},value=v${{ needs.detect.outputs.version }}
            type=raw,value=latest
      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
"#;

const RELEASE_WORKFLOW: &str = r#"name: Release
on:
  push:
    tags: ['v*']

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  github-release:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Install Rust
        uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --release --workspace
      - name: Create GitHub Release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          TAG="${{ github.ref_name }}"
          if gh release view "$TAG" &>/dev/null; then
            echo "Release $TAG already exists — skipping"
          else
            NOTES=""
            if [ -f CHANGELOG.md ]; then
              NOTES="--notes-file CHANGELOG.md"
            fi
            gh release create "$TAG" \
              --title "changelogger $TAG" \
              $NOTES
          fi

  crates-io:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@v2
      - name: Install changelogger
        run: cargo install changelogger-cli
      - name: Publish to crates.io
        run: changelogger publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

  container:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - name: Extract version from tag
        id: version
        run: echo "VERSION=${GITHUB_REF_NAME#v}" >> "$GITHUB_OUTPUT"
      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: .
          push: true
          tags: |
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ steps.version.outputs.VERSION }}
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest
          labels: |
            org.opencontainers.image.source=${{ github.server_url }}/${{ github.repository }}
            org.opencontainers.image.description=Changelogger — auto generate CHANGELOGs and prdocs
            org.opencontainers.image.licenses=Apache-2.0 OR MIT
          cache-from: type=gha
          cache-to: type=gha,mode=max
"#;
