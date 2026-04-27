mod claude;
mod codex;
mod flaude;
pub mod registry;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::ace_toml::Trust;
use crate::config::school_toml::McpDecl;

/// Everything a backend needs to launch a session.
///
/// `cmd` (the launch argv) is *not* in here — it's a property of the
/// backend instance, not session input. Per-backend `exec_session` takes
/// it as a separate parameter, populated by `Backend::exec_session` from
/// `self.cmd`.
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
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

    /// Canonical name. Doubles as registry key for built-in entries and as the
    /// default `cmd[0]` (the binary name) when no override is provided.
    pub fn name(&self) -> &'static str {
        match self {
            Kind::Claude => "claude",
            Kind::Codex => "codex",
            Kind::Flaude => "flaude",
        }
    }

    /// Lookup a built-in kind by canonical name.
    pub fn from_name(name: &str) -> Option<Kind> {
        Kind::ALL.iter().copied().find(|k| k.name() == name)
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

    pub fn exec_session(&self, cmd: &[String], opts: SessionOpts) -> Result<(), std::io::Error> {
        dispatch!(self, exec_session, cmd, opts)
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

/// Convert a `Kind` to its canonical name. Equivalent to
/// `k.name().to_string()`; provided so callsites can use `.into()` when
/// pushing a kind into a `String` field.
impl From<Kind> for String {
    fn from(k: Kind) -> String {
        k.name().to_string()
    }
}

/// Construct a `Backend` instance defaulted from a `Kind`: `name`/`cmd[0]`
/// = canonical name, empty env.
impl From<Kind> for Backend {
    fn from(kind: Kind) -> Backend {
        Backend {
            name: kind.name().to_string(),
            kind,
            cmd: vec![kind.name().to_string()],
            env: HashMap::new(),
        }
    }
}

impl Default for Backend {
    fn default() -> Backend {
        Kind::default().into()
    }
}

/// A resolved backend instance: identity (`name`), behavior (`kind`), and runtime
/// overrides (`cmd`, `env`). Built-ins are pre-built singletons; custom entries
/// from `[[backends]]` populate the registry alongside built-ins.
#[derive(Debug, Clone)]
pub struct Backend {
    pub name: String,
    pub kind: Kind,
    /// Argv for launching the binary. Built-ins seed `[kind.name()]`; custom
    /// backends from `[[backends]]` override.
    pub cmd: Vec<String>,
    pub env: HashMap<String, String>,
}

impl Backend {
    pub fn backend_dir(&self) -> &'static str {
        self.kind.backend_dir()
    }

    pub fn instructions_file(&self) -> &'static str {
        self.kind.instructions_file()
    }

    pub fn supports_trust(&self, trust: Trust) -> Result<(), String> {
        self.kind.supports_trust(trust)
    }

    pub fn exec_session(&self, mut opts: SessionOpts) -> Result<(), std::io::Error> {
        // per-backend env merges over global env (later wins on collision).
        for (k, v) in &self.env {
            opts.env.insert(k.clone(), v.clone());
        }
        self.kind.exec_session(&self.cmd, opts)
    }

    pub fn mcp_list(&self) -> HashSet<String> {
        self.kind.mcp_list()
    }

    pub fn mcp_remove(&self, name: &str) -> Result<(), String> {
        self.kind.mcp_remove(name)
    }

    pub fn mcp_check(&self, names: &[String]) -> Result<Vec<McpStatus>, String> {
        self.kind.mcp_check(names)
    }

    pub fn mcp_add(&self, entry: &McpDecl) -> Result<(), String> {
        self.kind.mcp_add(entry)
    }
}

/// Name → Backend lookup. Built with `Registry::with_builtins()`; layer-merge
/// from `[[backends]]` happens in `state::resolve_layers`.
#[derive(Debug, Default, Clone)]
pub struct Registry {
    entries: HashMap<String, Backend>,
}

impl Registry {
    pub fn with_builtins() -> Self {
        let entries = Kind::ALL.iter()
            .map(|k| (k.name().to_string(), Backend::from(*k)))
            .collect();
        Self { entries }
    }

    pub fn lookup(&self, name: &str) -> Option<&Backend> {
        self.entries.get(name)
    }

    pub(crate) fn get_mut(&mut self, name: &str) -> Option<&mut Backend> {
        self.entries.get_mut(name)
    }

    pub(crate) fn insert(&mut self, backend: Backend) {
        self.entries.insert(backend.name.clone(), backend);
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
