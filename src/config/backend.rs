use std::collections::HashSet;
use std::process::Command;

use serde::{Deserialize, Serialize};

use super::school_toml::McpDecl;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[default]
    Claude,
    OpenCode,
    Codex,
}

impl Backend {
    pub fn binary(&self) -> &'static str {
        match self {
            Backend::Claude => "claude",
            Backend::OpenCode => "opencode",
            Backend::Codex => "codex",
        }
    }

    pub fn backend_dir(&self) -> &'static str {
        match self {
            Backend::Claude => ".claude",
            Backend::OpenCode => ".opencode",
            Backend::Codex => ".agents",
        }
    }

    pub fn instructions_file(&self) -> &'static str {
        match self {
            Backend::Claude => "CLAUDE.md",
            Backend::OpenCode => "AGENTS.md",
            Backend::Codex => "AGENTS.md",
        }
    }

    /// List registered MCP server names. Best-effort: returns empty set on failure.
    pub fn mcp_list(&self) -> HashSet<String> {
        match self {
            Backend::Claude => claude_mcp_list(),
            Backend::OpenCode | Backend::Codex => HashSet::new(),
        }
    }

    /// Register an MCP server entry with the backend.
    pub fn mcp_add(&self, entry: &McpDecl) -> Result<(), String> {
        match self {
            Backend::Claude => claude_mcp_add(entry),
            Backend::OpenCode | Backend::Codex => {
                Err("MCP registration not yet implemented for this backend".to_string())
            }
        }
    }
}

// -- Claude backend MCP --

/// Read `~/.claude.json` and extract keys from the `mcpServers` object.
fn claude_mcp_list() -> HashSet<String> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return HashSet::new(),
    };

    let path = std::path::Path::new(&home).join(".claude.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };

    parse_claude_mcp_names(&content)
}

fn parse_claude_mcp_names(json: &str) -> HashSet<String> {
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return HashSet::new(),
    };

    parsed
        .get("mcpServers")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

fn claude_mcp_add(entry: &McpDecl) -> Result<(), String> {
    let mut args = vec![
        "mcp".to_string(),
        "add".to_string(),
        "-t".to_string(),
        "http".to_string(),
        "-s".to_string(),
        "user".to_string(),
    ];

    for (key, value) in &entry.headers {
        args.push("-H".to_string());
        args.push(format!("{key}: {value}"));
    }

    args.push(entry.name.clone());
    args.push(entry.url.clone());

    let output = Command::new("claude")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_claude_mcp_names_extracts_keys() {
        let json = r#"{
            "mcpServers": {
                "linear-server": {"type": "http", "url": "https://mcp.linear.app/mcp"},
                "github": {"type": "http", "url": "https://api.githubcopilot.com/mcp/"}
            }
        }"#;
        let names = parse_claude_mcp_names(json);
        assert_eq!(names.len(), 2);
        assert!(names.contains("linear-server"), "should contain linear-server");
        assert!(names.contains("github"), "should contain github");
    }

    #[test]
    fn parse_claude_mcp_names_missing_field() {
        let names = parse_claude_mcp_names(r#"{"something": "else"}"#);
        assert!(names.is_empty());
    }

    #[test]
    fn parse_claude_mcp_names_empty_servers() {
        let names = parse_claude_mcp_names(r#"{"mcpServers": {}}"#);
        assert!(names.is_empty());
    }

    #[test]
    fn parse_claude_mcp_names_invalid_json() {
        let names = parse_claude_mcp_names("not json");
        assert!(names.is_empty());
    }
}
