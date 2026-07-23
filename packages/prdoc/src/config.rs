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
    let config_path = root.join(".prdoc.toml");
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
            || current.join(".prdoc.toml").exists()
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
        let config_path = dir.path().join(".prdoc.toml");
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
