use std::path::Path;

use crate::config::ace_toml;
use crate::config::user_config::{self, SchoolEntry};
use crate::config::ConfigError;

pub struct WriteConfig;

impl WriteConfig {
    /// Create/update ~/.config/ace/config.toml with an entry for the school.
    /// Preserves existing entries and credentials — only adds the school key
    /// if it doesn't already exist.
    pub fn user(path: &Path, specifier: &str) -> Result<(), ConfigError> {
        let mut config = user_config::load_or_default(path)?;

        let repo_key = specifier
            .split_once(':')
            .map_or(specifier, |(repo, _)| repo);

        config
            .entry(repo_key.to_string())
            .or_insert_with(SchoolEntry::default);

        user_config::save(path, &config)
    }

    /// Write ace.toml with school = "<specifier>".
    /// Preserves existing env entries if the file already exists.
    pub fn project(path: &Path, specifier: &str) -> Result<(), ConfigError> {
        let mut config = ace_toml::load_or_default(path)?;
        config.school = specifier.to_string();
        ace_toml::save(path, &config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ace_toml::AceToml;
    use crate::config::user_config::UserConfig;

    #[test]
    fn project_creates_new_file() {
        let dir = std::env::temp_dir().join("ace-test-write-project");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("ace.toml");

        WriteConfig::project(&path, "prod9/school").expect("write project config");

        let content = std::fs::read_to_string(&path).expect("read ace.toml");
        let parsed: AceToml = toml::from_str(&content).expect("parse ace.toml");
        assert_eq!(parsed.school, "prod9/school");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn project_preserves_env() {
        let dir = std::env::temp_dir().join("ace-test-write-project-env");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("ace.toml");

        std::fs::write(&path, "school = \"old\"\n\n[env]\nKEY = \"value\"\n")
            .expect("write initial");

        WriteConfig::project(&path, "prod9/school").expect("write project config");

        let content = std::fs::read_to_string(&path).expect("read ace.toml");
        let parsed: AceToml = toml::from_str(&content).expect("parse ace.toml");
        assert_eq!(parsed.school, "prod9/school");
        assert_eq!(parsed.env.get("KEY").map(String::as_str), Some("value"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn user_creates_new_file() {
        let dir = std::env::temp_dir().join("ace-test-write-user");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("config.toml");

        WriteConfig::user(&path, "prod9/school").expect("write user config");

        let content = std::fs::read_to_string(&path).expect("read config.toml");
        let parsed: UserConfig = toml::from_str(&content).expect("parse config.toml");
        assert!(parsed.contains_key("prod9/school"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn user_preserves_existing_schools() {
        let dir = std::env::temp_dir().join("ace-test-write-user-preserve");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("config.toml");

        // Write first school
        WriteConfig::user(&path, "acme/school").expect("write first");
        // Write second school
        WriteConfig::user(&path, "prod9/school").expect("write second");

        let content = std::fs::read_to_string(&path).expect("read config.toml");
        let parsed: UserConfig = toml::from_str(&content).expect("parse config.toml");
        assert!(parsed.contains_key("acme/school"));
        assert!(parsed.contains_key("prod9/school"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn user_extracts_repo_from_specifier_with_path() {
        let dir = std::env::temp_dir().join("ace-test-write-user-repo");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("config.toml");

        WriteConfig::user(&path, "prod9/mono:school").expect("write user config");

        let content = std::fs::read_to_string(&path).expect("read config.toml");
        let parsed: UserConfig = toml::from_str(&content).expect("parse config.toml");
        assert!(parsed.contains_key("prod9/mono"), "key should be repo portion");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
