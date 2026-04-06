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

/// Dispatch a method call to the matching backend module's free function.
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            Backend::Claude => claude::$method($($arg),*),
            Backend::Codex => codex::$method($($arg),*),
            Backend::Droid => droid::$method($($arg),*),
            Backend::Flaude => flaude::$method($($arg),*),
            Backend::OpenCode => opencode::$method($($arg),*),
        }
    };
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

    pub fn exec_session(&self, opts: SessionOpts) -> Result<(), std::io::Error> {
        dispatch!(self, exec_session, opts)
    }

    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        dispatch!(self, is_ready)
    }

    pub fn mcp_list(&self) -> HashSet<String> {
        dispatch!(self, mcp_list)
    }

    pub fn mcp_remove(&self, name: &str) -> Result<(), String> {
        dispatch!(self, mcp_remove, name)
    }

    pub fn mcp_check(&self, names: &[String]) -> Result<Vec<McpStatus>, String> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        dispatch!(self, mcp_check, names)
    }

    pub fn mcp_add(&self, entry: &McpDecl) -> Result<(), String> {
        dispatch!(self, mcp_add, entry)
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
