mod claude;
mod codex;
mod droid;
mod flaude;
mod opencode;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::school_toml::McpDecl;

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

    /// Extra CLI args to skip permission prompts ("yolo mode").
    /// Returns an error message if the backend doesn't support it.
    pub fn yolo_args(&self) -> Result<Vec<String>, String> {
        match self {
            Backend::Claude => Ok(vec!["--dangerously-skip-permissions".to_string()]),
            Backend::Flaude => Ok(vec!["--yolo".to_string()]),
            Backend::Droid => Ok(vec!["--skip-permissions-unsafe".to_string()]),
            Backend::OpenCode | Backend::Codex => {
                Err(format!("yolo mode not supported for {}", self.binary()))
            }
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
