use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct AceToml {
    pub school: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

pub fn load(path: &Path) -> Result<AceToml, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: AceToml = toml::from_str(&content)?;
    Ok(config)
}
