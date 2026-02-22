use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::ConfigError;

#[derive(Debug, Deserialize, Serialize)]
pub struct AceToml {
    pub school: String,
    // TODO: add `role` and `description` fields so non-dev roles (e.g. PM) can
    // configure ace for requirements-only repos, prd/ workflows, Jira/Trello sync, etc.
    #[serde(default)]
    pub session_prompt: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

pub fn load(path: &Path) -> Result<AceToml, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: AceToml = toml::from_str(&content)?;
    Ok(config)
}
