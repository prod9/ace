mod claude;
mod codex;
mod droid;
mod flaude;
mod opencode;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::ace_toml::Trust;
use super::school_toml::McpDecl;

/// Everything a backend needs to launch a session.
pub struct SessionOpts {
    pub trust: Trust,
    pub session_prompt: String,
    pub project_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub extra_args: Vec<String>,
}

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

    /// Check whether the backend supports the given trust level.
    /// Returns Ok(()) if supported, Err(message) if not.
    pub fn supports_trust(&self, trust: Trust) -> Result<(), String> {
        match (self, trust) {
            (_, Trust::Default) => Ok(()),
            (Backend::Claude, Trust::Auto | Trust::Yolo) => Ok(()),
            (Backend::Flaude, Trust::Auto | Trust::Yolo) => Ok(()),
            (Backend::Droid, Trust::Yolo) => Ok(()),
            (Backend::Codex, Trust::Auto | Trust::Yolo) => Ok(()),
            (_, trust) => Err(format!(
                "trust={trust:?} not supported for {}",
                self.binary(),
            )),
        }
    }

    /// Launch a backend session. Each backend builds its own Command internally.
    /// Flaude records to JSONL instead of exec'ing (test fake).
    pub fn exec_session(&self, opts: SessionOpts) -> Result<(), std::io::Error> {
        match self {
            Backend::Claude => claude::exec_session(opts),
            Backend::Codex => codex::exec_session(opts),
            Backend::Droid => droid::exec_session(opts),
            Backend::Flaude => flaude::exec_session(opts),
            Backend::OpenCode => opencode::exec_session(opts),
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
    fn supports_trust_default_all() {
        for backend in [Backend::Claude, Backend::Flaude, Backend::Droid, Backend::OpenCode, Backend::Codex] {
            backend.supports_trust(Trust::Default)
                .unwrap_or_else(|_| panic!("{:?} should support Default", backend));
        }
    }

    #[test]
    fn supports_trust_auto_claude() {
        Backend::Claude.supports_trust(Trust::Auto).expect("Claude supports Auto");
    }

    #[test]
    fn supports_trust_yolo_claude() {
        Backend::Claude.supports_trust(Trust::Yolo).expect("Claude supports Yolo");
    }

    #[test]
    fn supports_trust_auto_flaude() {
        Backend::Flaude.supports_trust(Trust::Auto).expect("Flaude supports Auto");
    }

    #[test]
    fn supports_trust_yolo_flaude() {
        Backend::Flaude.supports_trust(Trust::Yolo).expect("Flaude supports Yolo");
    }

    #[test]
    fn supports_trust_yolo_droid() {
        Backend::Droid.supports_trust(Trust::Yolo).expect("Droid supports Yolo");
    }

    #[test]
    fn supports_trust_auto_codex() {
        Backend::Codex.supports_trust(Trust::Auto).expect("Codex supports Auto");
    }

    #[test]
    fn supports_trust_yolo_codex() {
        Backend::Codex.supports_trust(Trust::Yolo).expect("Codex supports Yolo");
    }

    #[test]
    fn supports_trust_auto_droid_unsupported() {
        let err = Backend::Droid.supports_trust(Trust::Auto).expect_err("Droid should not support Auto");
        assert!(err.contains("droid"), "error should mention the backend name");
    }

    #[test]
    fn supports_trust_auto_opencode_unsupported() {
        let err = Backend::OpenCode.supports_trust(Trust::Auto).expect_err("OpenCode should not support Auto");
        assert!(err.contains("opencode"), "error should mention the backend name");
    }
}
