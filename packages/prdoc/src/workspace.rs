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

use cargo_metadata::MetadataCommand;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub crates: Vec<WorkspaceCrate>,
    pub crate_by_path: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceCrate {
    pub name: String,
    pub manifest_path: PathBuf,
    pub publish: bool,
}

impl WorkspaceInfo {
    pub fn load_from_root(root: &Path) -> Result<Self, WorkspaceError> {
        let metadata = MetadataCommand::new()
            .manifest_path(root.join("Cargo.toml"))
            .exec()?;

        let mut crates = Vec::new();
        let mut crate_by_path = HashMap::new();

        for pkg in &metadata.workspace_packages() {
            let pkg_path =
                pkg.manifest_path.parent().unwrap_or(&pkg.manifest_path);
            let pkg_path = pkg_path.as_std_path();
            let is_publish =
                pkg.publish.as_ref().map(|p| !p.is_empty()).unwrap_or(true);

            let crate_info = WorkspaceCrate {
                name: pkg.name.clone(),
                manifest_path: pkg.manifest_path.as_std_path().to_path_buf(),
                publish: is_publish,
            };

            let path_str = pkg_path
                .strip_prefix(root)
                .unwrap_or(pkg_path)
                .to_string_lossy()
                .trim_start_matches('/')
                .to_string();
            crate_by_path.insert(path_str, pkg.name.clone());
            crates.push(crate_info);
        }

        Ok(WorkspaceInfo {
            root: root.to_path_buf(),
            crates,
            crate_by_path,
        })
    }

    pub fn find_crate_for_path(&self, file_path: &str) -> Option<String> {
        let mut best_match: Option<(&String, &String)> = None;
        for (crate_path, name) in &self.crate_by_path {
            if file_path.starts_with(crate_path) {
                let is_better = match best_match {
                    Some((existing, _)) => crate_path.len() > existing.len(),
                    None => true,
                };
                if is_better {
                    best_match = Some((crate_path, name));
                }
            }
        }
        best_match.map(|(_, name)| name.clone())
    }

    pub fn crate_names(&self) -> Vec<String> {
        let mut names: Vec<String> =
            self.crates.iter().map(|c| c.name.clone()).collect();
        names.sort();
        names
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("Cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),
    #[error("No Cargo.toml found at {0}")]
    NotFound(String),
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_crate_for_path_matches_longest_prefix() {
        let mut info = WorkspaceInfo {
            root: PathBuf::from("/test"),
            crates: vec![],
            crate_by_path: HashMap::new(),
        };
        info.crate_by_path
            .insert("/test/packages/core".to_string(), "core".to_string());
        info.crate_by_path
            .insert("/test/packages/core/sub".to_string(), "sub".to_string());

        assert_eq!(
            info.find_crate_for_path("/test/packages/core/sub/foo.rs"),
            Some("sub".to_string())
        );
        assert_eq!(
            info.find_crate_for_path("/test/packages/core/src/lib.rs"),
            Some("core".to_string())
        );
        assert_eq!(info.find_crate_for_path("/test/other/file.rs"), None);
    }
}
