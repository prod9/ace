use std::collections::HashSet;
use std::path::PathBuf;

use super::McpDecl;

/// Returns OpenCode's data directory (~/.local/share/opencode or $OPENCODE_HOME).
#[allow(dead_code)]
fn data_dir() -> PathBuf {
    std::env::var("OPENCODE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("opencode")
        })
}

/// Returns OpenCode's config directory (~/.config/opencode).
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencode")
}

/// Check if OpenCode is ready: auth.json exists and is non-empty.
pub(super) fn is_ready() -> bool {
    let auth_path = data_dir().join("auth.json");
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

/// Read MCP server names from ~/.config/opencode/opencode.json.
/// Extracts keys from the "mcp" object where type is "remote".
pub(super) fn mcp_list() -> HashSet<String> {
    let config_path = config_dir().join("opencode.json");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };
    parse_mcp_names(&content)
}

fn parse_mcp_names(json: &str) -> HashSet<String> {
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return HashSet::new(),
    };

    let Some(mcp_obj) = parsed.get("mcp").and_then(|v| v.as_object()) else {
        return HashSet::new();
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

/// Remove an MCP server entry from ~/.config/opencode/opencode.json.
pub(super) fn mcp_remove(name: &str) -> Result<(), String> {
    use std::io::Write;

    let config_path = config_dir().join("opencode.json");

    let existing = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .map_err(|e| format!("read {}: {e}", config_path.display()))?
    } else {
        return Ok(());
    };

    let output = remove_mcp_entry(&existing, name)?;

    let mut file = std::fs::File::create(&config_path)
        .map_err(|e| format!("create {}: {e}", config_path.display()))?;
    file.write_all(output.as_bytes())
        .map_err(|e| format!("write {}: {e}", config_path.display()))?;

    Ok(())
}

/// Pure logic: remove an MCP entry from an existing OpenCode config JSON string.
fn remove_mcp_entry(existing_json: &str, name: &str) -> Result<String, String> {
    let mut config: serde_json::Value =
        serde_json::from_str(existing_json).map_err(|e| format!("parse config: {e}"))?;

    if let Some(mcp) = config.get_mut("mcp").and_then(|v| v.as_object_mut()) {
        mcp.remove(name);
    }

    serde_json::to_string_pretty(&config).map_err(|e| format!("serialize config: {e}"))
}

/// Add an MCP server entry to ~/.config/opencode/opencode.json.
/// Merges into existing config, preserving other entries.
pub(super) fn mcp_add(entry: &McpDecl) -> Result<(), String> {
    use std::io::Write;

    let config_dir = config_dir();
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("create {}: {e}", config_dir.display()))?;

    let config_path = config_dir.join("opencode.json");

    let existing = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .map_err(|e| format!("read {}: {e}", config_path.display()))?
    } else {
        "{}".to_string()
    };

    let output = merge_mcp_entry(&existing, entry)?;

    let mut file = std::fs::File::create(&config_path)
        .map_err(|e| format!("create {}: {e}", config_path.display()))?;
    file.write_all(output.as_bytes())
        .map_err(|e| format!("write {}: {e}", config_path.display()))?;

    Ok(())
}

/// Pure logic: merge an MCP entry into an existing OpenCode config JSON string.
/// Returns the updated JSON string (pretty-printed).
fn merge_mcp_entry(existing_json: &str, entry: &McpDecl) -> Result<String, String> {
    let mut config: serde_json::Value =
        serde_json::from_str(existing_json).map_err(|e| format!("parse config: {e}"))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mcp_names_extracts_remote() {
        let json = r#"{
            "mcp": {
                "linear": {"type": "remote", "url": "https://mcp.linear.app/mcp"},
                "github": {"type": "remote", "url": "https://api.githubcopilot.com/mcp/"},
                "local-tool": {"type": "local", "command": ["npx", "tool"]}
            }
        }"#;
        let names = parse_mcp_names(json);
        assert_eq!(names.len(), 2);
        assert!(names.contains("linear"), "should contain linear");
        assert!(names.contains("github"), "should contain github");
        assert!(!names.contains("local-tool"), "should not contain local MCP");
    }

    #[test]
    fn parse_mcp_names_missing_field() {
        let names = parse_mcp_names(r#"{"something": "else"}"#);
        assert!(names.is_empty());
    }

    #[test]
    fn parse_mcp_names_empty_mcp() {
        let names = parse_mcp_names(r#"{"mcp": {}}"#);
        assert!(names.is_empty());
    }

    #[test]
    fn parse_mcp_names_invalid_json() {
        let names = parse_mcp_names("not json");
        assert!(names.is_empty());
    }

    #[test]
    fn merge_into_empty() {
        let entry = McpDecl {
            name: "linear".to_string(),
            url: "https://mcp.linear.app/mcp".to_string(),
            headers: std::collections::HashMap::new(),
            instructions: String::new(),
        };

        let result = merge_mcp_entry("{}", &entry).expect("should merge");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse");
        assert_eq!(parsed["mcp"]["linear"]["type"], "remote");
        assert_eq!(parsed["mcp"]["linear"]["url"], "https://mcp.linear.app/mcp");
    }

    #[test]
    fn merge_preserves_existing() {
        let existing = r#"{"model": "claude", "mcp": {"github": {"type": "remote", "url": "https://github.com/mcp"}}}"#;

        let entry = McpDecl {
            name: "linear".to_string(),
            url: "https://mcp.linear.app/mcp".to_string(),
            headers: std::collections::HashMap::new(),
            instructions: String::new(),
        };

        let result = merge_mcp_entry(existing, &entry).expect("should merge");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse");

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

    // -- remove_mcp_entry --

    #[test]
    fn remove_existing_entry() {
        let existing = r#"{"mcp": {"linear": {"type": "remote", "url": "https://mcp.linear.app/mcp"}, "github": {"type": "remote", "url": "https://github.com/mcp"}}}"#;
        let result = remove_mcp_entry(existing, "linear").expect("should remove");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse");

        assert!(parsed["mcp"]["linear"].is_null(), "linear should be removed");
        assert_eq!(parsed["mcp"]["github"]["type"], "remote", "github should remain");
    }

    #[test]
    fn remove_missing_entry_is_ok() {
        let existing = r#"{"mcp": {"linear": {"type": "remote", "url": "https://mcp.linear.app/mcp"}}}"#;
        let result = remove_mcp_entry(existing, "nonexistent").expect("should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse");
        assert_eq!(parsed["mcp"]["linear"]["type"], "remote", "linear should remain");
    }

    #[test]
    fn remove_preserves_non_mcp_fields() {
        let existing = r#"{"model": "claude", "mcp": {"linear": {"type": "remote", "url": "https://mcp.linear.app/mcp"}}}"#;
        let result = remove_mcp_entry(existing, "linear").expect("should remove");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse");

        assert_eq!(parsed["model"], "claude", "should preserve non-mcp fields");
    }

    #[test]
    fn remove_from_no_mcp_section() {
        let result = remove_mcp_entry(r#"{"model": "claude"}"#, "linear").expect("should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse");
        assert_eq!(parsed["model"], "claude");
    }

    #[test]
    fn merge_with_headers() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let entry = McpDecl {
            name: "sentry".to_string(),
            url: "https://mcp.sentry.dev/mcp".to_string(),
            headers,
            instructions: String::new(),
        };

        let result = merge_mcp_entry("{}", &entry).expect("should merge");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("should parse");

        assert_eq!(
            parsed["mcp"]["sentry"]["headers"]["Authorization"],
            "Bearer token123"
        );
    }
}
