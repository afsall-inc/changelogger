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
        ChangelogCmd::Generate { from, output, dir } => {
            cmd_changelog_generate(from.as_deref(), &output, dir.as_deref())
        }
        ChangelogCmd::Bump {
            current,
            from,
            dry_run,
        } => cmd_changelog_bump(&current, from.as_deref(), dry_run),
        ChangelogCmd::Verify { from } => cmd_changelog_verify(from.as_deref()),
    }
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
        let schema = include_str!("../../../../prdoc/schema_user.json");
        std::fs::write(&schema_path, schema).map_err(|e| format!("{e}"))?;
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
        let prdocs = changelogger_prdoc::load_prdocs_from_dir_recursive(&path);
        let mut all_issues = Vec::new();
        for prdoc in &prdocs {
            let issues = if let Some(branch_name) = branch {
                changelogger_prdoc::validate_prdoc_for_branch(
                    prdoc,
                    branch_name,
                )
            } else {
                changelogger_prdoc::validate_prdoc(prdoc)
            };
            if !issues.is_empty() {
                all_issues.push(format!(
                    "Issues in prdoc:\n{}",
                    issues.join("\n  - ")
                ));
            }
        }
        if all_issues.is_empty() {
            Ok(format!("Validated {} prdoc file(s).", prdocs.len()))
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

    let rendered = changelog.render();
    std::fs::write(output, &rendered).map_err(|e| e.to_string())?;

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
