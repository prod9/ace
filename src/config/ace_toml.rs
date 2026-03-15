use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::backend::Backend;
use super::{is_empty_str, is_empty_map, ConfigError};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AceToml {
    #[serde(skip_serializing_if = "is_empty_str")]
    pub school: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<Backend>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_prompt: Option<String>,
    #[serde(skip_serializing_if = "is_empty_map")]
    pub env: HashMap<String, String>,
}

pub fn load(path: &Path) -> Result<AceToml, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: AceToml = toml::from_str(&content)?;
    Ok(config)
}

pub fn save(path: &Path, toml: &AceToml) -> Result<(), ConfigError> {
    let content = toml::to_string_pretty(toml)?;
    std::fs::write(path, content)?;
    Ok(())
}
