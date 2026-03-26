use std::collections::HashSet;

use super::McpDecl;

#[allow(dead_code)]
pub(super) fn is_ready() -> bool {
    false
}

pub(super) fn mcp_list() -> HashSet<String> {
    HashSet::new()
}

pub(super) fn mcp_add(_entry: &McpDecl) -> Result<(), String> {
    Err("MCP registration not yet implemented for this backend".to_string())
}
