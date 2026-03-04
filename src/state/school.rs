use serde::Serialize;
use std::collections::HashMap;

use crate::config::school_toml::{ImportDecl, McpDecl, Project, SchoolToml};

#[derive(Debug, Default, Serialize)]
pub struct School {
    pub name: String,
    pub session_prompt: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mcp: Vec<McpDecl>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<Project>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<ImportDecl>,
}

impl From<SchoolToml> for School {
    fn from(st: SchoolToml) -> Self {
        Self {
            name: st.name,
            session_prompt: st.session_prompt,
            env: st.env,
            mcp: st.mcp,
            projects: st.projects,
            imports: st.imports,
        }
    }
}
