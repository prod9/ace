use std::collections::HashSet;
use std::path::PathBuf;

use super::McpDecl;

/// Returns Codex's home directory (`$CODEX_HOME` or `~/.codex`).
fn home_dir() -> Option<PathBuf> {
    std::env::var("CODEX_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".codex")))
}

/// Check if Codex is ready: auth.json exists OR API key env var is set.
#[allow(dead_code)]
pub(super) fn is_ready() -> bool {
    std::env::var("CODEX_API_KEY").is_ok()
        || std::env::var("OPENAI_API_KEY").is_ok()
        || home_dir()
            .map(|d| d.join("auth.json").exists())
            .unwrap_or(false)
}

pub(super) fn mcp_list() -> HashSet<String> {
    HashSet::new()
}

pub(super) fn mcp_add(_entry: &McpDecl) -> Result<(), String> {
    Err("MCP registration not yet implemented for this backend".to_string())
}
