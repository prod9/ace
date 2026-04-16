use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::backend::Backend;
use super::{is_empty_str, is_empty_map, ConfigError};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Trust {
    #[default]
    Default,
    Auto,
    Yolo,
}

impl Trust {
    pub fn is_default(&self) -> bool {
        matches!(self, Trust::Default)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AceToml {
    #[serde(skip_serializing_if = "is_empty_str")]
    pub school: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<Backend>,
    // TODO: add `role` and `description` fields so non-dev roles (e.g. PM) can
    // configure ace for requirements-only repos, spec/ workflows, Jira/Trello sync, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_prompt: Option<String>,
    #[serde(skip_serializing_if = "is_empty_map")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Trust::is_default")]
    pub trust: Trust,

    /// Auto-resume previous session. Personal-only (local config).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<bool>,

    /// Disable automatic version checks and background upgrades.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_update: Option<bool>,

    /// Deprecated: use `trust = "yolo"` instead. Kept for backcompat.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub yolo: bool,
}

pub fn load(path: &Path) -> Result<AceToml, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: AceToml = toml::from_str(&content)?;
    Ok(config)
}

/// Load from file, returning default if the file doesn't exist.
/// Errors on invalid TOML or other I/O failures.
pub fn load_or_default(path: &Path) -> Result<AceToml, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(toml::from_str(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AceToml::default()),
        Err(e) => Err(ConfigError::from(e)),
    }
}

pub fn save(path: &Path, toml: &AceToml) -> Result<(), ConfigError> {
    let content = toml::to_string_pretty(toml)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Set the school specifier, preserving all other fields.
pub fn set_school(path: &Path, specifier: &str) -> Result<(), ConfigError> {
    let mut config = load_or_default(path)?;
    config.school = specifier.to_string();
    save(path, &config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_or_default_missing_file() {
        let path = std::path::Path::new("/tmp/ace-test-nonexistent/ace.toml");
        let result = load_or_default(path).expect("should return default");
        assert!(result.school.is_empty());
        assert!(result.backend.is_none());
    }

    #[test]
    fn load_or_default_existing_file() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("ace.toml");
        std::fs::write(&path, "school = \"prod9/school\"\n").expect("write");

        let result = load_or_default(&path).expect("should load");
        assert_eq!(result.school, "prod9/school");
    }

    #[test]
    fn load_or_default_invalid_toml() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("ace.toml");
        std::fs::write(&path, "not valid {{{{ toml").expect("write");

        assert!(load_or_default(&path).is_err());
    }

    #[test]
    fn set_school_creates_new_file() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("ace.toml");

        set_school(&path, "prod9/school").expect("set school");

        let config = load(&path).expect("reload");
        assert_eq!(config.school, "prod9/school");
    }

    #[test]
    fn set_school_preserves_env() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("ace.toml");
        std::fs::write(&path, "school = \"old\"\n\n[env]\nKEY = \"value\"\n").expect("write");

        set_school(&path, "prod9/school").expect("set school");

        let config = load(&path).expect("reload");
        assert_eq!(config.school, "prod9/school");
        assert_eq!(config.env.get("KEY").map(String::as_str), Some("value"));
    }
}
