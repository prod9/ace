use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::ConfigError;

/// ~/.config/ace/config.toml
///
/// Top-level keys are school identifiers ("owner/repo").
/// Each school has a `services` map of service name -> credentials.
///
/// ```toml
/// ["acme-corp/school".services.github]
/// token = "gho_..."
///
/// ["acme-corp/school".services.jira]
/// token = "eyJ..."
/// username = "alice"
/// ```
pub type UserConfig = HashMap<String, SchoolCredentials>;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SchoolCredentials {
    pub services: HashMap<String, ServiceCredentials>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceCredentials {
    pub token: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

pub fn load(path: &Path) -> Result<UserConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: UserConfig = toml::from_str(&content)?;
    Ok(config)
}

pub fn default_path() -> Option<std::path::PathBuf> {
    dirs_or_home().map(|p| p.join("ace").join("config.toml"))
}

fn dirs_or_home() -> Option<std::path::PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config"))
        })
}
