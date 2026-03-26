use std::collections::HashSet;

use super::McpDecl;

/// Check if DROID is ready: ~/.factory/settings.json exists.
pub(super) fn is_ready() -> bool {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };
    std::path::Path::new(&home)
        .join(".factory/settings.json")
        .exists()
}

pub(super) fn mcp_list() -> HashSet<String> {
    // Factory backend MCP list not yet fully implemented - minimal stub to allow compilation
    // Real implementation would read from ~/.factory/mcp.json
    HashSet::new()
}

pub(super) fn mcp_add(_entry: &McpDecl) -> Result<(), String> {
    // Factory backend MCP add not yet fully implemented - minimal stub to allow compilation
    // Real implementation would call: droid mcp add <name> <url> --type http --header "K:V"
    Err("Droid backend MCP registration not yet implemented".to_string())
}
