use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct SchoolToml {
    pub school: SchoolMeta,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub services: Vec<ServiceDecl>,
    #[serde(default)]
    pub mcp: Vec<McpDecl>,
    #[serde(default)]
    pub projects: Vec<Project>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SchoolMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceDecl {
    pub name: String,
    pub authorize_url: String,
    pub token_url: String,
    pub client_id: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct McpDecl {
    pub name: String,
    pub image: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub repo: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub mcp: Vec<McpDecl>,
}

pub fn load(path: &Path) -> Result<SchoolToml, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: SchoolToml = toml::from_str(&content)?;
    Ok(config)
}
