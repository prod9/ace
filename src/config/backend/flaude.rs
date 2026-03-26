use std::collections::HashSet;

use super::McpDecl;

/// Read `FLAUDE_MCP_LIST` env var as comma-separated server names.
pub(super) fn mcp_list() -> HashSet<String> {
    std::env::var("FLAUDE_MCP_LIST")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Append a JSON line to the file at `FLAUDE_RECORD`.
pub(super) fn mcp_add(entry: &McpDecl) -> Result<(), String> {
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
