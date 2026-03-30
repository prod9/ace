use std::collections::HashSet;
use std::path::PathBuf;

use super::McpDecl;

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
    let path = match mcp_list_path() {
        Some(p) => p,
        None => return HashSet::new(),
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
