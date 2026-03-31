use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::ConfigError;

/// ~/.config/ace/config.toml
///
/// Top-level keys are school identifiers ("owner/repo").
///
/// ```toml
/// ["acme-corp/school"]
///
/// ["myuser/school"]
/// ```
pub type UserConfig = HashMap<String, SchoolEntry>;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SchoolEntry {}

/// Load from file, returning empty config if the file doesn't exist.
/// Errors on invalid TOML or other I/O failures.
pub fn load_or_default(path: &Path) -> Result<UserConfig, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(toml::from_str(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(UserConfig::default()),
        Err(e) => Err(ConfigError::from(e)),
    }
}

pub fn save(path: &Path, config: &UserConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| ConfigError::Encode(e))?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_or_default_missing_file() {
        let path = Path::new("/tmp/ace-test-nonexistent/config.toml");
        let result = load_or_default(path).expect("should return default");
        assert!(result.is_empty());
    }

    #[test]
    fn load_or_default_existing_file() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[\"prod9/school\"]\n").expect("write");

        let result = load_or_default(&path).expect("should load");
        assert!(result.contains_key("prod9/school"));
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path().join("nested/dir/config.toml");

        let mut config = UserConfig::new();
        config.insert("prod9/school".to_string(), SchoolEntry::default());

        save(&path, &config).expect("should save");

        let loaded = load_or_default(&path).expect("should reload");
        assert!(loaded.contains_key("prod9/school"));
    }
}
