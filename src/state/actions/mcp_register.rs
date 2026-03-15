use crate::ace::Ace;
use crate::config::backend::Backend;
use crate::config::school_toml::McpDecl;

#[derive(Debug, thiserror::Error)]
pub enum McpRegisterError {
    #[error("mcp register failed: {0}")]
    Register(String),
}

pub struct McpRegister<'a> {
    pub backend: Backend,
    pub entries: &'a [McpDecl],
}

impl McpRegister<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<(), McpRegisterError> {
        if self.entries.is_empty() {
            return Ok(());
        }

        ace.progress("Checking MCP servers...");
        let registered = self.backend.mcp_list();

        for entry in self.entries {
            if registered.contains(&entry.name) {
                continue;
            }

            self.backend.mcp_add(entry)
                .map_err(|e| McpRegisterError::Register(format!("{}: {e}", entry.name)))?;

            if entry.headers.is_empty() {
                ace.done(&format!(
                    "Registered MCP server '{}' — you'll be prompted to authorize on first use",
                    entry.name,
                ));
            } else {
                ace.done(&format!("Registered MCP server '{}'", entry.name));
            }
        }

        Ok(())
    }
}
