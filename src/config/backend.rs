use std::collections::HashSet;
use std::path::PathBuf;
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
    Flaude,
    Droid,
}

impl Backend {
    pub fn binary(&self) -> &'static str {
        match self {
            Backend::Claude => "claude",
            Backend::OpenCode => "opencode",
            Backend::Codex => "codex",
            Backend::Flaude => "flaude",
            Backend::Droid => "droid",
        }
    }

    pub fn backend_dir(&self) -> &'static str {
        match self {
            Backend::Claude | Backend::Flaude => ".claude",
            Backend::OpenCode => ".opencode",
            Backend::Codex => ".agents",
            Backend::Droid => ".factory",
        }
    }

    pub fn instructions_file(&self) -> &'static str {
        match self {
            Backend::Claude | Backend::Flaude => "CLAUDE.md",
            Backend::OpenCode => "AGENTS.md",
            Backend::Codex => "AGENTS.md",
            Backend::Droid => "AGENTS.md",
        }
    }

    // TODO: Re-analyze the abstraction boundary between ACE and backends. Currently
    // ACE knows backend-specific flags (yolo, system-prompt, etc.) scattered across
    // Exec and here. Consider whether backends should own their full arg construction
    // (a BackendOpts struct or trait) instead of ACE assembling args piecemeal.

    /// Extra CLI args to skip permission prompts ("yolo mode").
    /// Returns an error message if the backend doesn't support it.
    pub fn yolo_args(&self) -> Result<Vec<String>, String> {
        match self {
            Backend::Claude => Ok(vec!["--dangerously-skip-permissions".to_string()]),
            Backend::Flaude => Ok(vec!["--yolo".to_string()]),
            Backend::Droid => Ok(vec!["--skip-permissions-unsafe".to_string()]),
            Backend::OpenCode | Backend::Codex => {
                Err(format!("yolo mode not supported for {}", self.binary()))
            }
        }
    }

    /// Check if the backend is ready to use (authenticated/configured).
    /// Returns true if the backend appears to be set up, false otherwise.
    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        match self {
            Backend::Claude => claude_is_ready(),
            Backend::Flaude => true,
            Backend::Droid => droid_is_ready(),
            Backend::OpenCode => opencode_is_ready(),
            Backend::Codex => false,
        }
    }

    /// List registered MCP server names. Best-effort: returns empty set on failure.
    pub fn mcp_list(&self) -> HashSet<String> {
        match self {
            Backend::Claude => claude_mcp_list(),
            Backend::Flaude => flaude_mcp_list(),
            Backend::Droid => droid_mcp_list(),
            Backend::OpenCode => opencode_mcp_list(),
            Backend::Codex => HashSet::new(),
        }
    }

    /// Register an MCP server entry with the backend.
    pub fn mcp_add(&self, entry: &McpDecl) -> Result<(), String> {
        match self {
            Backend::Claude => claude_mcp_add(entry),
            Backend::Flaude => flaude_mcp_add(entry),
            Backend::Droid => droid_mcp_add(entry),
            Backend::OpenCode => opencode_mcp_add(entry),
            Backend::Codex => {
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

// -- Flaude test backend --
//
// Records MCP calls to a JSONL file instead of spawning a real backend binary.
// Used by integration tests with `backend = "flaude"` in ace.toml.

/// Read `FLAUDE_MCP_LIST` env var as comma-separated server names.
fn flaude_mcp_list() -> HashSet<String> {
    std::env::var("FLAUDE_MCP_LIST")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Append a JSON line to the file at `FLAUDE_RECORD`.
fn flaude_mcp_add(entry: &McpDecl) -> Result<(), String> {
    use std::io::Write;

    let record_path =
        std::env::var("FLAUDE_RECORD").map_err(|_| "FLAUDE_RECORD env var not set".to_string())?;

    let mut headers: Vec<(&String, &String)> = entry.headers.iter().collect();
    headers.sort_by_key(|(k, _)| k.as_str());

    let record = serde_json::json!({
        "action": "mcp_add",
        "name": entry.name,
        "url": entry.url,
        "headers": headers.iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>(),
    });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&record_path)
        .map_err(|e| format!("open {record_path}: {e}"))?;

    writeln!(file, "{record}").map_err(|e| format!("write {record_path}: {e}"))?;
    Ok(())
}

// -- Claude backend readiness --

#[allow(dead_code)]
fn claude_is_ready() -> bool {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };
    std::path::Path::new(&home).join(".claude.json").exists()
}

// -- OpenCode backend --

/// Returns OpenCode's data directory (~/.local/share/opencode or $OPENCODE_HOME).
#[allow(dead_code)]
fn opencode_data_dir() -> PathBuf {
    std::env::var("OPENCODE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("opencode")
        })
}

/// Returns OpenCode's config directory (~/.config/opencode).
fn opencode_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencode")
}

/// Check if OpenCode is ready: auth.json exists and is non-empty.
fn opencode_is_ready() -> bool {
    let auth_path = opencode_data_dir().join("auth.json");
    if !auth_path.exists() {
        return false;
    }
    match std::fs::read_to_string(&auth_path) {
        Ok(content) => {
            let trimmed = content.trim();
            trimmed != "{}" && !trimmed.is_empty()
        }
        Err(_) => false,
    }
}

// Factory backend functions
fn droid_mcp_list() -> HashSet<String> {
    // Factory backend MCP list not yet fully implemented - minimal stub to allow compilation
    // This is a placeholder - real implementation would read from ~/.factory/mcp.json
    HashSet::new()
}

fn droid_mcp_add(_entry: &McpDecl) -> Result<(), String> {
    // Factory backend MCP add not yet fully implemented - minimal stub to allow compilation
    // Real implementation would call: droid mcp add <name> <url> --type http --header "K:V"
    Err("Droid backend MCP registration not yet implemented".to_string())
}

/// Read MCP server names from ~/.config/opencode/opencode.json.
/// Extracts keys from the "mcp" object where type is "remote".
fn opencode_mcp_list() -> HashSet<String> {
    let config_path = opencode_config_dir().join("opencode.json");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };
    parse_opencode_mcp_names(&content)
}

fn parse_opencode_mcp_names(json: &str) -> HashSet<String> {
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return HashSet::new(),
    };

    let mcp_obj = match parsed.get("mcp").and_then(|v| v.as_object()) {
        Some(obj) => obj,
        None => return HashSet::new(),
    };

    mcp_obj
        .iter()
        .filter(|(_, v)| {
            v.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "remote")
                .unwrap_or(false)
        })
        .map(|(k, _)| k.clone())
        .collect()
}

/// Add an MCP server entry to ~/.config/opencode/opencode.json.
/// Merges into existing config, preserving other entries.
fn opencode_mcp_add(entry: &McpDecl) -> Result<(), String> {
    use std::io::Write;

    let config_dir = opencode_config_dir();
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("create {}: {e}", config_dir.display()))?;

    let config_path = config_dir.join("opencode.json");

    let existing = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .map_err(|e| format!("read {}: {e}", config_path.display()))?
    } else {
        "{}".to_string()
    };

    let output = merge_opencode_mcp_entry(&existing, entry)?;

    let mut file = std::fs::File::create(&config_path)
        .map_err(|e| format!("create {}: {e}", config_path.display()))?;
    file.write_all(output.as_bytes())
        .map_err(|e| format!("write {}: {e}", config_path.display()))?;

    Ok(())
}

/// Pure logic: merge an MCP entry into an existing OpenCode config JSON string.
/// Returns the updated JSON string (pretty-printed).
fn merge_opencode_mcp_entry(existing_json: &str, entry: &McpDecl) -> Result<String, String> {
    let mut config: serde_json::Value = serde_json::from_str(existing_json)
        .map_err(|e| format!("parse config: {e}"))?;

    let mcp = config
        .as_object_mut()
        .ok_or("config root is not an object")?
        .entry("mcp".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("mcp is not an object")?;

    let mut server_entry = serde_json::json!({
        "type": "remote",
        "url": entry.url,
    });

    if !entry.headers.is_empty() {
        let mut sorted_headers: Vec<(&String, &String)> = entry.headers.iter().collect();
        sorted_headers.sort_by_key(|(k, _)| k.as_str());
        server_entry["headers"] = serde_json::to_value(
            sorted_headers
                .into_iter()
                .collect::<std::collections::BTreeMap<_, _>>(),
        )
        .map_err(|e| format!("serialize headers: {e}"))?;
    }

    mcp.insert(entry.name.clone(), server_entry);

    serde_json::to_string_pretty(&config).map_err(|e| format!("serialize config: {e}"))
}

/// Check if DROID is ready: ~/.factory/settings.json exists.
fn droid_is_ready() -> bool {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };
    std::path::Path::new(&home)
        .join(".factory/settings.json")
        .exists()
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
        assert!(
            names.contains("linear-server"),
            "should contain linear-server"
        );
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

    // -- OpenCode MCP tests --

    #[test]
    fn parse_opencode_mcp_names_extracts_remote() {
        let json = r#"{
            "mcp": {
                "linear": {"type": "remote", "url": "https://mcp.linear.app/mcp"},
                "github": {"type": "remote", "url": "https://api.githubcopilot.com/mcp/"},
                "local-tool": {"type": "local", "command": ["npx", "tool"]}
            }
        }"#;
        let names = parse_opencode_mcp_names(json);
        assert_eq!(names.len(), 2);
        assert!(names.contains("linear"), "should contain linear");
        assert!(names.contains("github"), "should contain github");
        assert!(
            !names.contains("local-tool"),
            "should not contain local MCP"
        );
    }

    #[test]
    fn parse_opencode_mcp_names_missing_field() {
        let names = parse_opencode_mcp_names(r#"{"something": "else"}"#);
        assert!(names.is_empty());
    }

    #[test]
    fn parse_opencode_mcp_names_empty_mcp() {
        let names = parse_opencode_mcp_names(r#"{"mcp": {}}"#);
        assert!(names.is_empty());
    }

    #[test]
    fn parse_opencode_mcp_names_invalid_json() {
        let names = parse_opencode_mcp_names("not json");
        assert!(names.is_empty());
    }

    #[test]
    fn merge_opencode_mcp_into_empty() {
        let entry = McpDecl {
            name: "linear".to_string(),
            url: "https://mcp.linear.app/mcp".to_string(),
            headers: std::collections::HashMap::new(),
            instructions: String::new(),
        };

        let result = merge_opencode_mcp_entry("{}", &entry).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["mcp"]["linear"]["type"], "remote");
        assert_eq!(parsed["mcp"]["linear"]["url"], "https://mcp.linear.app/mcp");
    }

    #[test]
    fn merge_opencode_mcp_preserves_existing() {
        let existing = r#"{"model": "claude", "mcp": {"github": {"type": "remote", "url": "https://github.com/mcp"}}}"#;

        let entry = McpDecl {
            name: "linear".to_string(),
            url: "https://mcp.linear.app/mcp".to_string(),
            headers: std::collections::HashMap::new(),
            instructions: String::new(),
        };

        let result = merge_opencode_mcp_entry(existing, &entry).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["model"], "claude", "should preserve existing fields");
        assert_eq!(
            parsed["mcp"]["github"]["type"], "remote",
            "should preserve existing MCP"
        );
        assert_eq!(
            parsed["mcp"]["linear"]["type"], "remote",
            "should add new MCP"
        );
    }

    #[test]
    fn merge_opencode_mcp_with_headers() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let entry = McpDecl {
            name: "sentry".to_string(),
            url: "https://mcp.sentry.dev/mcp".to_string(),
            headers,
            instructions: String::new(),
        };

        let result = merge_opencode_mcp_entry("{}", &entry).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(
            parsed["mcp"]["sentry"]["headers"]["Authorization"],
            "Bearer token123"
        );
    }
}
