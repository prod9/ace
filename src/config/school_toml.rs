use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::backend::Backend;
use super::{is_empty_str, is_empty_map, is_empty_vec, is_false, ConfigError};

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
    #[serde(skip_serializing_if = "is_false")]
    pub include_experimental: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub include_system: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct McpDecl {
    pub name: String,
    pub url: String,
    #[serde(skip_serializing_if = "is_empty_map")]
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub instructions: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Project {
    pub name: String,
    pub repo: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub description: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_decl_default_flags_false() {
        let toml_str = "skill = \"foo\"\nsource = \"owner/repo\"\n";
        let decl: ImportDecl = toml::from_str(toml_str).expect("parse");
        assert_eq!(decl.skill, "foo");
        assert_eq!(decl.source, "owner/repo");
        assert!(!decl.include_experimental);
        assert!(!decl.include_system);
    }

    #[test]
    fn import_decl_parses_include_flags() {
        let toml_str = "skill = \"*\"\nsource = \"owner/repo\"\ninclude_experimental = true\ninclude_system = true\n";
        let decl: ImportDecl = toml::from_str(toml_str).expect("parse");
        assert!(decl.include_experimental);
        assert!(decl.include_system);
    }

    #[test]
    fn import_decl_omits_false_flags_when_serialized() {
        let decl = ImportDecl {
            skill: "foo".to_string(),
            source: "owner/repo".to_string(),
            include_experimental: false,
            include_system: false,
        };
        let out = toml::to_string(&decl).expect("serialize");
        assert!(!out.contains("include_experimental"),
            "false include_experimental should not be serialized: {out}");
        assert!(!out.contains("include_system"),
            "false include_system should not be serialized: {out}");
    }

    #[test]
    fn import_decl_writes_true_flags() {
        let decl = ImportDecl {
            skill: "*".to_string(),
            source: "owner/repo".to_string(),
            include_experimental: true,
            include_system: false,
        };
        let out = toml::to_string(&decl).expect("serialize");
        assert!(out.contains("include_experimental = true"), "missing flag in {out}");
        assert!(!out.contains("include_system"), "false flag should be omitted: {out}");
    }
}
