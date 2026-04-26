mod claude;
mod codex;
mod flaude;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::config::ace_toml::Trust;
use crate::config::school_toml::McpDecl;

/// Everything a backend needs to launch a session.
pub struct SessionOpts {
    pub trust: Trust,
    pub session_prompt: String,
    pub project_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub extra_args: Vec<String>,
    pub resume: bool,
}

/// Health check result for a single MCP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpStatus {
    pub name: String,
    pub ok: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Claude,
    Codex,
    Flaude,
}

/// Dispatch a method call to the matching backend module's free function.
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            Kind::Claude => claude::$method($($arg),*),
            Kind::Codex => codex::$method($($arg),*),
            Kind::Flaude => flaude::$method($($arg),*),
        }
    };
}

impl Kind {
    pub const ALL: &'static [Kind] = &[Kind::Claude, Kind::Codex, Kind::Flaude];

    pub fn binary(&self) -> &'static str {
        match self {
            Kind::Claude => "claude",
            Kind::Codex => "codex",
            Kind::Flaude => "flaude",
        }
    }

    pub fn backend_dir(&self) -> &'static str {
        match self {
            Kind::Claude | Kind::Flaude => ".claude",
            Kind::Codex => ".agents",
        }
    }

    pub fn instructions_file(&self) -> &'static str {
        match self {
            Kind::Claude | Kind::Flaude => "CLAUDE.md",
            Kind::Codex => "AGENTS.md",
        }
    }

    pub fn supports_trust(&self, _trust: Trust) -> Result<(), String> {
        // All current backends (Claude, Codex, Flaude) support all trust levels.
        Ok(())
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

/// A resolved backend instance: identity (`name`), behavior (`kind`), and runtime
/// overrides (`cmd`, `env`). Built-ins are pre-built singletons; custom entries
/// from `[[backends]]` populate the registry alongside built-ins.
#[derive(Debug, Clone)]
#[allow(dead_code)] // cmd + env wired in subsequent slice
pub struct Backend {
    pub name: String,
    pub kind: Kind,
    pub cmd: Vec<String>,
    pub env: HashMap<String, String>,
}

/// Name → Backend lookup. Built with `Registry::with_builtins()` then extended
/// with parsed `[[backends]]` entries from each config layer.
#[derive(Debug, Default, Clone)]
#[allow(dead_code)] // wired into State in subsequent slice
pub struct Registry {
    entries: HashMap<String, Backend>,
}

#[allow(dead_code)] // wired into State in subsequent slice
impl Registry {
    pub fn with_builtins() -> Self {
        let mut entries = HashMap::new();
        for kind in Kind::ALL {
            let name = kind.binary().to_string();
            entries.insert(name.clone(), Backend {
                name,
                kind: *kind,
                cmd: vec![kind.binary().to_string()],
                env: HashMap::new(),
            });
        }
        Self { entries }
    }

    pub fn lookup(&self, name: &str) -> Option<&Backend> {
        self.entries.get(name)
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
    use super::{Kind, Registry};
    use crate::config::ace_toml::Trust;

    #[test]
    fn supports_trust_default_all() {
        for backend in [Kind::Claude, Kind::Flaude, Kind::Codex] {
            backend.supports_trust(Trust::Default)
                .unwrap_or_else(|_| panic!("{:?} should support Default", backend));
        }
    }

    #[test]
    fn supports_trust_auto_claude() {
        Kind::Claude.supports_trust(Trust::Auto).expect("Claude supports Auto");
    }

    #[test]
    fn supports_trust_yolo_claude() {
        Kind::Claude.supports_trust(Trust::Yolo).expect("Claude supports Yolo");
    }

    #[test]
    fn supports_trust_auto_flaude() {
        Kind::Flaude.supports_trust(Trust::Auto).expect("Flaude supports Auto");
    }

    #[test]
    fn supports_trust_yolo_flaude() {
        Kind::Flaude.supports_trust(Trust::Yolo).expect("Flaude supports Yolo");
    }

    #[test]
    fn supports_trust_auto_codex() {
        Kind::Codex.supports_trust(Trust::Auto).expect("Codex supports Auto");
    }

    #[test]
    fn supports_trust_yolo_codex() {
        Kind::Codex.supports_trust(Trust::Yolo).expect("Codex supports Yolo");
    }

    #[test]
    fn registry_with_builtins_lookup() {
        let registry = Registry::with_builtins();

        let claude = registry.lookup("claude").expect("claude builtin");
        assert_eq!(claude.kind, Kind::Claude);
        assert_eq!(claude.name, "claude");

        let codex = registry.lookup("codex").expect("codex builtin");
        assert_eq!(codex.kind, Kind::Codex);

        let flaude = registry.lookup("flaude").expect("flaude builtin");
        assert_eq!(flaude.kind, Kind::Flaude);

        assert!(registry.lookup("unknown").is_none());
    }
}
