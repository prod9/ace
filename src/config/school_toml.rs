use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::backend::Backend;
use super::{is_empty_str, is_empty_map, is_empty_vec, ConfigError};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SchoolToml {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<Backend>,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub session_prompt: String,
    #[serde(skip_serializing_if = "is_empty_map")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub mcp: Vec<McpDecl>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub projects: Vec<Project>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub imports: Vec<ImportDecl>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ImportDecl {
    pub skill: String,
    pub source: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct McpDecl {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Project {
    pub name: String,
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "is_empty_map")]
    pub env: HashMap<String, String>,
}

pub fn load(path: &Path) -> Result<SchoolToml, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: SchoolToml = toml::from_str(&content)?;
    Ok(config)
}

pub fn save(path: &Path, toml: &SchoolToml) -> Result<(), ConfigError> {
    let content = toml::to_string_pretty(toml)?;
    std::fs::write(path, content)?;
    Ok(())
}
