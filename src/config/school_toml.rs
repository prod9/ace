use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::backend::Backend;
use super::ConfigError;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SchoolToml {
    pub school: SchoolMeta,
    pub env: HashMap<String, String>,
    pub services: Vec<ServiceDecl>,
    pub mcp: Vec<McpDecl>,
    pub projects: Vec<Project>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SchoolMeta {
    pub name: String,
    pub description: Option<String>,
    pub backend: Option<Backend>,
    pub session_prompt: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ServiceDecl {
    pub name: String,
    pub authorize_url: String,
    pub token_url: String,
    pub client_id: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct McpDecl {
    pub name: String,
    pub image: String,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Project {
    pub name: String,
    pub repo: String,
    pub description: Option<String>,
    pub env: HashMap<String, String>,
    pub mcp: Vec<McpDecl>,
}

pub fn load(path: &Path) -> Result<SchoolToml, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: SchoolToml = toml::from_str(&content)?;
    Ok(config)
}
