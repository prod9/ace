use crate::ace::Ace;
use crate::config::backend::Backend;

pub struct RemoveMcp<'a> {
    pub backend: Backend,
    pub names: &'a [String],
}

impl RemoveMcp<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<(), String> {
        for name in self.names {
            match self.backend.mcp_remove(name) {
                Ok(()) => ace.done(&format!("Removed MCP server '{name}'")),
                Err(e) => ace.warn(&format!("Failed to remove '{name}': {e}")),
            }
        }
        Ok(())
    }
}
