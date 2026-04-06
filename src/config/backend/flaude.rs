use std::collections::HashSet;
use std::path::PathBuf;

use super::{McpDecl, McpStatus};

/// Flaude record file for MCP registrations.
fn mcp_record_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::Path::new(&h).join(".flaude-mcp-records.jsonl")
    })
}

/// Flaude MCP list file — one server name per line.
fn mcp_list_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::Path::new(&h).join(".flaude-mcp-list")
    })
}

/// Read registered MCP names from `$HOME/.flaude-mcp-list` (one per line).
pub(super) fn mcp_list() -> HashSet<String> {
    let Some(path) = mcp_list_path() else {
        return HashSet::new();
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };

    content
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Test fake: returns all servers as healthy.
pub(super) fn mcp_check(names: &[String]) -> Vec<McpStatus> {
    names.iter()
        .map(|n| McpStatus { name: n.clone(), ok: true })
        .collect()
}

/// Remove an MCP server by name. Appends a removal record and updates the list file.
pub(super) fn mcp_remove(name: &str) -> Result<(), String> {
    use std::io::Write;

    // -- append removal record --

    let record_path = mcp_record_path()
        .ok_or_else(|| "HOME not set".to_string())?;

    let record = serde_json::json!({
        "action": "mcp_remove",
        "name": name,
    });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&record_path)
        .map_err(|e| format!("open {}: {e}", record_path.display()))?;

    writeln!(file, "{record}").map_err(|e| format!("write {}: {e}", record_path.display()))?;

    // -- update list file --

    let Some(list_path) = mcp_list_path() else {
        return Ok(());
    };

    let existing = std::fs::read_to_string(&list_path).unwrap_or_default();
    let updated: Vec<&str> = existing
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != name)
        .collect();

    std::fs::write(&list_path, updated.join("\n"))
        .map_err(|e| format!("write {}: {e}", list_path.display()))?;

    Ok(())
}

/// Append a JSON record to `$HOME/.flaude-mcp-records.jsonl`.
pub(super) fn mcp_add(entry: &McpDecl) -> Result<(), String> {
    use std::io::Write;

    let record_path = mcp_record_path()
        .ok_or_else(|| "HOME not set".to_string())?;

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
        .map_err(|e| format!("open {}: {e}", record_path.display()))?;

    writeln!(file, "{record}").map_err(|e| format!("write {}: {e}", record_path.display()))?;
    Ok(())
}
