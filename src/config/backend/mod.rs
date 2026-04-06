mod claude;
mod codex;
mod droid;
mod flaude;
mod opencode;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::school_toml::McpDecl;

/// Health check result for a single MCP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpStatus {
    pub name: String,
    pub ok: bool,
}

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

    /// Extra CLI args for the given trust level.
    /// Returns an error message if the backend doesn't support it.
    pub fn trust_args(&self, trust: super::ace_toml::Trust) -> Result<Vec<String>, String> {
        use super::ace_toml::Trust;
        match (self, trust) {
            (_, Trust::Default) => Ok(vec![]),
            (Backend::Claude, Trust::Auto) => Ok(vec![
                "--permission-mode".to_string(), "auto".to_string(),
            ]),
            (Backend::Claude, Trust::Yolo) => Ok(vec![
                "--permission-mode".to_string(), "bypassPermissions".to_string(),
            ]),
            (Backend::Flaude, Trust::Auto) => Ok(vec!["--auto".to_string()]),
            (Backend::Flaude, Trust::Yolo) => Ok(vec!["--yolo".to_string()]),
            (Backend::Droid, Trust::Yolo) => Ok(vec![
                "--skip-permissions-unsafe".to_string(),
            ]),
            (_, trust) => Err(format!(
                "trust={trust:?} not supported for {}",
                self.binary(),
            )),
        }
    }

    /// Check if the backend is ready to use (authenticated/configured).
    /// Returns true if the backend appears to be set up, false otherwise.
    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        match self {
            Backend::Claude => claude::is_ready(),
            Backend::Flaude => true,
            Backend::Droid => droid::is_ready(),
            Backend::OpenCode => opencode::is_ready(),
            Backend::Codex => codex::is_ready(),
        }
    }

    /// List registered MCP server names. Best-effort: returns empty set on failure.
    pub fn mcp_list(&self) -> HashSet<String> {
        match self {
            Backend::Claude => claude::mcp_list(),
            Backend::Flaude => flaude::mcp_list(),
            Backend::Droid => droid::mcp_list(),
            Backend::OpenCode => opencode::mcp_list(),
            Backend::Codex => codex::mcp_list(),
        }
    }

    /// Remove a registered MCP server by name.
    pub fn mcp_remove(&self, name: &str) -> Result<(), String> {
        match self {
            Backend::Claude => claude::mcp_remove(name),
            Backend::Flaude => flaude::mcp_remove(name),
            Backend::Droid => droid::mcp_remove(name),
            Backend::OpenCode => opencode::mcp_remove(name),
            Backend::Codex => codex::mcp_remove(name),
        }
    }

    /// Health-check registered MCP servers via one-shot backend prompt.
    /// Returns Ok(statuses) on success, Err(reason) when the check itself fails.
    pub fn mcp_check(&self, names: &[String]) -> Result<Vec<McpStatus>, String> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        match self {
            Backend::Claude => claude::mcp_check(names),
            Backend::OpenCode => opencode::mcp_check(names),
            Backend::Droid => droid::mcp_check(names),
            Backend::Flaude => flaude::mcp_check(names),
            Backend::Codex => codex::mcp_check(names),
        }
    }

    /// Register an MCP server entry with the backend.
    pub fn mcp_add(&self, entry: &McpDecl) -> Result<(), String> {
        match self {
            Backend::Claude => claude::mcp_add(entry),
            Backend::Flaude => flaude::mcp_add(entry),
            Backend::Droid => droid::mcp_add(entry),
            Backend::OpenCode => opencode::mcp_add(entry),
            Backend::Codex => codex::mcp_add(entry),
        }
    }
}

/// Parse `[{"name":"...","ok":bool}]` JSON into McpStatus vec.
/// Shared helper — each backend extracts the JSON string from its own output format,
/// then calls this to parse the common shape.
pub(super) fn parse_status_array(json: &str) -> Vec<McpStatus> {
    #[derive(serde::Deserialize)]
    struct Entry {
        name: String,
        ok: bool,
    }

    let entries: Vec<Entry> = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    entries.into_iter()
        .map(|e| McpStatus { name: e.name, ok: e.ok })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::Backend;
    use crate::config::ace_toml::Trust;

    #[test]
    fn trust_default_returns_empty() {
        for backend in [Backend::Claude, Backend::Flaude, Backend::Droid, Backend::OpenCode, Backend::Codex] {
            let args = backend.trust_args(Trust::Default).expect("Default should always succeed");
            assert!(args.is_empty(), "{:?} should return empty vec for Default", backend);
        }
    }

    #[test]
    fn trust_auto_claude() {
        let args = Backend::Claude.trust_args(Trust::Auto).expect("Claude supports Auto");
        assert_eq!(args, vec!["--permission-mode", "auto"]);
    }

    #[test]
    fn trust_yolo_claude() {
        let args = Backend::Claude.trust_args(Trust::Yolo).expect("Claude supports Yolo");
        assert_eq!(args, vec!["--permission-mode", "bypassPermissions"]);
    }

    #[test]
    fn trust_yolo_flaude() {
        let args = Backend::Flaude.trust_args(Trust::Yolo).expect("Flaude supports Yolo");
        assert_eq!(args, vec!["--yolo"]);
    }

    #[test]
    fn trust_auto_flaude() {
        let args = Backend::Flaude.trust_args(Trust::Auto).expect("Flaude supports Auto");
        assert_eq!(args, vec!["--auto"]);
    }

    #[test]
    fn trust_yolo_droid() {
        let args = Backend::Droid.trust_args(Trust::Yolo).expect("Droid supports Yolo");
        assert_eq!(args, vec!["--skip-permissions-unsafe"]);
    }

    #[test]
    fn trust_auto_unsupported() {
        let err = Backend::Droid.trust_args(Trust::Auto).expect_err("Droid should not support Auto");
        assert!(err.contains("droid"), "error should mention the backend name");
    }

    #[test]
    fn trust_auto_opencode_unsupported() {
        let err = Backend::OpenCode.trust_args(Trust::Auto).expect_err("OpenCode should not support Auto");
        assert!(err.contains("opencode"), "error should mention the backend name");
    }
}
