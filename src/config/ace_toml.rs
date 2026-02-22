use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::backend::Backend;
use super::ConfigError;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct AceToml {
    pub school: String,
    pub backend: Option<Backend>,
    // TODO: add `role` and `description` fields so non-dev roles (e.g. PM) can
    // configure ace for requirements-only repos, prd/ workflows, Jira/Trello sync, etc.
    pub session_prompt: Option<String>,
    pub env: HashMap<String, String>,
}

pub fn load(path: &Path) -> Result<AceToml, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: AceToml = toml::from_str(&content)?;
    Ok(config)
}
