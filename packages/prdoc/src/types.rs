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

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrDoc {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<u64>,
    pub doc: Vec<DocSection>,
    pub crates: Vec<CrateChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrations: Option<Migrations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_functions: Option<Vec<HostFunction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Audience {
    #[serde(rename = "Developer")]
    Developer,
    #[serde(rename = "User")]
    User,
    #[serde(rename = "Operator")]
    Operator,
}

impl Audience {
    pub fn as_str(&self) -> &'static str {
        match self {
            Audience::Developer => "Developer",
            Audience::User => "User",
            Audience::Operator => "Operator",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        let normalized = s.to_lowercase().replace("-", "").replace("_", "");
        match normalized.as_str() {
            "developer" | "dev" => Audience::Developer,
            "user" => Audience::User,
            "operator" | "op" => Audience::Operator,
            _ => Audience::Developer,
        }
    }

    pub fn all() -> &'static [Audience] {
        &[Audience::Developer, Audience::User, Audience::Operator]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSection {
    pub audience: Audience,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrateChange {
    pub name: String,
    pub bump: BumpLevel,
    #[serde(
        default = "default_validate",
        skip_serializing_if = "is_default_validate"
    )]
    pub validate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn default_validate() -> bool {
    true
}

fn is_default_validate(v: &bool) -> bool {
    *v
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BumpLevel {
    Major,
    Minor,
    Patch,
    None,
}

impl BumpLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            BumpLevel::Major => "major",
            BumpLevel::Minor => "minor",
            BumpLevel::Patch => "patch",
            BumpLevel::None => "none",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "major" => BumpLevel::Major,
            "minor" => BumpLevel::Minor,
            "patch" => BumpLevel::Patch,
            _ => BumpLevel::None,
        }
    }

    pub fn dominates(&self, other: &BumpLevel) -> bool {
        let ord = |l: &BumpLevel| match l {
            BumpLevel::None => 0,
            BumpLevel::Patch => 1,
            BumpLevel::Minor => 2,
            BumpLevel::Major => 3,
        };
        ord(self) >= ord(other)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Migrations {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub db: Vec<DbMigration>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub runtime: Vec<RuntimeMigration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMigration {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMigration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostFunction {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PrdocError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid frontmatter: {0}")]
    Frontmatter(String),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub fn parse_prdoc(content: &str) -> Result<PrDoc, PrdocError> {
    let frontmatter = extract_frontmatter(content)?;
    serde_yaml::from_str(&frontmatter).map_err(PrdocError::Yaml)
}

pub fn load_prdoc(path: &Path) -> Result<PrDoc, PrdocError> {
    let content = std::fs::read_to_string(path)?;
    parse_prdoc(&content)
}

pub fn validate_prdoc(prdoc: &PrDoc) -> Vec<String> {
    let mut issues = Vec::new();

    if prdoc.title.is_empty() || prdoc.title == "..." {
        issues.push("title is required and cannot be '...'".to_string());
    }

    if prdoc.doc.is_empty() {
        issues.push("at least one doc section is required".to_string());
    }

    for (i, doc) in prdoc.doc.iter().enumerate() {
        if doc.description.is_empty() || doc.description == "..." {
            issues.push(format!(
                "doc[{}].description is required and cannot be '...'",
                i
            ));
        }
    }

    if prdoc.crates.is_empty() {
        issues.push("at least one crate must be listed".to_string());
    }

    for crate_change in &prdoc.crates {
        if crate_change.name.is_empty() {
            issues.push("crate name must not be empty".to_string());
        }
    }

    issues
}

pub fn validate_prdoc_for_branch(prdoc: &PrDoc, branch: &str) -> Vec<String> {
    let mut issues = validate_prdoc(prdoc);

    if branch.starts_with("stable") || branch.starts_with("release") {
        for crate_change in &prdoc.crates {
            if crate_change.bump == BumpLevel::Major && crate_change.validate {
                issues.push(format!(
                    "crate '{}' has major bump on backport branch '{}' but \
                     validate=true. Set validate: false if intentional.",
                    crate_change.name, branch
                ));
            }
        }
    }

    issues
}

fn extract_frontmatter(content: &str) -> Result<String, PrdocError> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(PrdocError::Frontmatter(
            "content must start with YAML frontmatter (---)".to_string(),
        ));
    }
    let end = trimmed[3..].find("\n---").ok_or(PrdocError::Frontmatter(
        "frontmatter must be closed with ---".to_string(),
    ))?;
    Ok(trimmed[3..end + 3].trim().to_string())
}

/// Fix common issues in a prdoc file content in-place.
/// Returns list of fixes applied.
pub fn fix_prdoc_content(content: &mut String) -> Vec<String> {
    let mut fixes = Vec::new();

    // Fix empty crates section: `crates: []` or `crates: [ ]` or `crates:\n`
    let empty_crates_re =
        regex::Regex::new(r"(?m)^crates:\s*\[\s*\]\s*$").unwrap();
    if empty_crates_re.is_match(content) {
        *content = empty_crates_re
            .replace(
                content,
                "crates:\n  - name: changelogger-cli\n    bump: patch",
            )
            .to_string();
        fixes.push(
            "replaced empty crates section with default crate".to_string(),
        );
    }

    // Fix missing crates section entirely (no `- name:` anywhere)
    if !content.contains("- name:") {
        let trimmed = content.trim_end();
        if let Some(stripped) = trimmed.strip_suffix("---") {
            let base = stripped.trim_end().to_string();
            *content = format!(
                "{}\ncrates:\n  - name: changelogger-cli\n    bump: \
                 patch\n---\n",
                base
            );
        } else {
            *content = format!(
                "{}\ncrates:\n  - name: changelogger-cli\n    bump: patch\n",
                trimmed
            );
        }
        fixes.push(
            "added missing crates section with default crate".to_string(),
        );
    }

    // Fix placeholder descriptions
    if content.contains("description: |\n      ...") {
        *content = content.replace("...", "auto-fixed: see PR description");
        fixes.push("replaced placeholder description".to_string());
    }

    fixes
}

/// Load, fix, and re-validate a prdoc file. Returns the fixed PrDoc and list of fixes.
pub fn load_and_fix_prdoc(
    path: &Path,
) -> Result<(PrDoc, Vec<String>), PrdocError> {
    let mut content = std::fs::read_to_string(path)?;
    let fixes = fix_prdoc_content(&mut content);
    if !fixes.is_empty() {
        std::fs::write(path, &content).map_err(PrdocError::Io)?;
    }
    let prdoc = parse_prdoc(&content)?;
    Ok((prdoc, fixes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_prdoc() {
        let content = r#"---
title: "Add new feature"
doc:
  - audience: Developer
    description: "A new feature"
crates:
  - name: my-crate
    bump: minor
---"#;
        let prdoc = parse_prdoc(content).unwrap();
        assert_eq!(prdoc.title, "Add new feature");
        assert_eq!(prdoc.crates.len(), 1);
        assert_eq!(prdoc.crates[0].name, "my-crate");
        assert_eq!(prdoc.crates[0].bump, BumpLevel::Minor);
    }

    #[test]
    fn validate_empty_title() {
        let prdoc = PrDoc::default();
        let issues = validate_prdoc(&prdoc);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("title")));
    }

    #[test]
    fn bump_level_ordering() {
        assert!(BumpLevel::Major.dominates(&BumpLevel::Minor));
        assert!(BumpLevel::Minor.dominates(&BumpLevel::Patch));
        assert!(BumpLevel::Patch.dominates(&BumpLevel::None));
        assert!(!BumpLevel::None.dominates(&BumpLevel::Patch));
    }
}
