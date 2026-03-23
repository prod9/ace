use std::collections::{HashMap, HashSet};

use crate::ace::{Ace, IoError};
use crate::config::backend::Backend;
use crate::config::school_toml::McpDecl;
use crate::templates::Template;

#[derive(Debug, thiserror::Error)]
pub enum McpRegisterError {
    #[error("mcp register failed: {0}")]
    Register(String),
    #[error("{0}")]
    Io(#[from] IoError),
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

        // -- check which servers are already registered --

        ace.progress("Checking MCP servers...");
        let registered = self.backend.mcp_list();

        // -- register missing servers --

        for entry in self.entries {
            if registered.contains(&entry.name) {
                continue;
            }

            let resolved = resolve_headers(entry, ace)?;
            let target = resolved.as_ref().unwrap_or(entry);

            self.backend.mcp_add(target)
                .map_err(|e| McpRegisterError::Register(format!("{}: {e}", entry.name)))?;

            let msg = registration_message(&entry.name, entry.headers.is_empty());
            ace.done(&msg);
        }

        Ok(())
    }
}

fn registration_message(name: &str, no_headers: bool) -> String {
    if no_headers {
        format!("Registered MCP server '{name}' — you'll be prompted to authorize on first use")
    } else {
        format!("Registered MCP server '{name}'")
    }
}

/// Parse header values for `{{ placeholder }}` syntax, prompt the user, and return
/// a resolved copy. Returns `None` if no placeholders were found.
fn resolve_headers(entry: &McpDecl, ace: &mut Ace) -> Result<Option<McpDecl>, IoError> {
    // -- collect unique placeholders --

    let all_placeholders = collect_placeholders(&entry.headers);

    if all_placeholders.is_empty() {
        return Ok(None);
    }

    // -- prompt for values --

    if !entry.instructions.is_empty() {
        ace.hint(&entry.instructions);
    }

    let mut values = HashMap::new();
    for name in &all_placeholders {
        let input = ace.prompt_text(&format!("{} ({}):", name, entry.name), None)?;
        values.insert(name.clone(), input);
    }

    // -- substitute into headers --

    let resolved_headers = entry.headers.iter()
        .map(|(k, v)| {
            let tpl = Template::parse(v);
            (k.clone(), tpl.substitute(&values))
        })
        .collect();

    Ok(Some(McpDecl {
        name: entry.name.clone(),
        url: entry.url.clone(),
        headers: resolved_headers,
        instructions: entry.instructions.clone(),
    }))
}

fn collect_placeholders(headers: &HashMap<String, String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut placeholders = Vec::new();

    for value in headers.values() {
        for name in Template::parse(value).placeholders() {
            if seen.insert(name.to_string()) {
                placeholders.push(name.to_string());
            }
        }
    }

    placeholders
}
