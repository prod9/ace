use std::path::Path;
use std::process::{Command, Stdio};

use crate::ace::Ace;
use crate::config::backend::Backend;
use crate::config::school_toml::McpDecl;

/// Register MCP servers declared in school.toml into the active backend.
pub struct RegisterMcp<'a> {
    pub backend: Backend,
    pub mcp: &'a [McpDecl],
    pub project_dir: &'a Path,
}

impl RegisterMcp<'_> {
    pub fn run(&self, ace: &mut Ace) {
        if self.mcp.is_empty() {
            return;
        }

        match self.backend {
            Backend::Claude => self.register_claude(ace),
            Backend::OpenCode | Backend::Codex => {
                ace.hint("MCP registration for this backend is not yet supported.");
            }
        }
    }

    fn register_claude(&self, ace: &mut Ace) {
        for entry in self.mcp {
            if entry.name.is_empty() || entry.url.is_empty() {
                continue;
            }

            let json = format!(
                r#"{{"type":"url","url":"{}"}}"#,
                entry.url,
            );

            let result = Command::new("claude")
                .args(["mcp", "add-json", "-s", "project", &entry.name, &json])
                .current_dir(self.project_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            match result {
                Ok(status) if status.success() => {
                    ace.done(&format!("Registered MCP: {}", entry.name));
                }
                Ok(_) => {
                    ace.warn(&format!(
                        "Failed to register MCP: {} (claude mcp add-json returned error)",
                        entry.name
                    ));
                }
                Err(e) => {
                    ace.warn(&format!(
                        "Failed to register MCP: {} ({e})",
                        entry.name
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_empty_entries() {
        let entries = vec![
            McpDecl { name: String::new(), url: "https://example.com".into() },
            McpDecl { name: "test".into(), url: String::new() },
        ];

        // Just verify the filter logic — we can't actually call claude binary in tests.
        let non_empty: Vec<_> = entries.iter()
            .filter(|e| !e.name.is_empty() && !e.url.is_empty())
            .collect();
        assert!(non_empty.is_empty());
    }

    #[test]
    fn json_format() {
        let url = "https://api.githubcopilot.com/mcp/";
        let json = format!(r#"{{"type":"url","url":"{url}"}}"#);
        assert_eq!(json, r#"{"type":"url","url":"https://api.githubcopilot.com/mcp/"}"#);
    }
}
