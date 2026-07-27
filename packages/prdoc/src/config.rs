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
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrdocConfig {
    #[serde(default)]
    pub version: u32,
    #[serde(default = "default_schema")]
    pub schema: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_prdoc_folders")]
    pub prdoc_folders: Vec<String>,
    #[serde(default = "default_template")]
    pub template: String,
    #[serde(default)]
    pub audiences: Option<AudiencesConfig>,
    #[serde(default)]
    pub generate: GenerateSection,
}

impl Default for PrdocConfig {
    fn default() -> Self {
        PrdocConfig {
            version: 0,
            schema: default_schema(),
            output_dir: default_output_dir(),
            prdoc_folders: default_prdoc_folders(),
            template: default_template(),
            audiences: None,
            generate: GenerateSection::default(),
        }
    }
}

fn default_schema() -> String {
    "prdoc/schema_user.json".to_string()
}

fn default_output_dir() -> String {
    "prdoc".to_string()
}

fn default_prdoc_folders() -> Vec<String> {
    vec!["prdoc".to_string()]
}

fn default_template() -> String {
    "templates/prdoc/.template.prdoc".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudiencesConfig {
    #[serde(default)]
    pub developer: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub operator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerateSection {
    #[serde(default)]
    pub default_output: String,
}

pub fn load_config(root: &Path) -> PrdocConfig {
    let config_path = root.join("changelogger.toml");
    if !config_path.exists() {
        return PrdocConfig::default();
    }

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return PrdocConfig::default(),
    };

    toml::from_str(&content).unwrap_or_default()
}

pub fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join("Cargo.toml").exists()
            || current.join("changelogger.toml").exists()
        {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_default_config() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_config(dir.path());
        assert_eq!(config.version, 0);
        assert_eq!(config.output_dir, "prdoc");
    }

    #[test]
    fn load_custom_config() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("changelogger.toml");
        fs::write(
            &config_path,
            r#"
version = 1
schema = "schemas/custom.json"
output_dir = "docs/prdoc"
prdoc_folders = ["prdoc"]
template = "prdoc.template"

[audiences]
developer = "Framework Dev"
user = "App Dev"
operator = "Operator"

[generate]
default_output = "CHANGELOG.md"
"#,
        )
        .unwrap();

        let config = load_config(dir.path());
        assert_eq!(config.version, 1);
        assert_eq!(config.schema, "schemas/custom.json");
        assert_eq!(config.output_dir, "docs/prdoc");
        assert_eq!(config.generate.default_output, "CHANGELOG.md");
    }
}
