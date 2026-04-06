use std::collections::HashSet;

use clap::Subcommand;

use crate::ace::Ace;
use crate::config::school_toml::McpDecl;
use crate::state::actions::mcp_register::McpRegister;
use crate::state::actions::mcp_remove::McpRemove;

use super::CmdError;

#[derive(Subcommand)]
pub enum Command {
    /// Health-check registered MCP servers (read-only)
    Check,
    /// Remove registered MCP servers, then re-add with `ace mcp`
    Reset {
        /// Specific server name to remove (omit for all school-defined)
        name: Option<String>,
    },
    /// Remove registered MCP servers (alias for reset)
    #[command(hide = true)]
    Clear {
        /// Specific server name to remove (omit for all school-defined)
        name: Option<String>,
    },
}

pub fn run(ace: &mut Ace, command: Option<Command>) {
    let result = match command {
        None => run_default(ace),
        Some(Command::Check) => run_check(ace),
        Some(Command::Reset { name } | Command::Clear { name }) => run_reset(ace, name),
    };
    super::exit_on_err(ace, result);
}

/// `ace mcp` — check health, add missing, prompt to re-register broken.
fn run_default(ace: &mut Ace) -> Result<(), CmdError> {
    ace.require_state()?;

    let (backend, entries) = load_school_mcp(ace)?;
    if entries.is_empty() {
        ace.hint("no MCP servers defined in school");
        return Ok(());
    }

    let registered = backend.mcp_list();

    // -- add missing --

    let missing: Vec<_> = entries.iter()
        .filter(|e| !registered.contains(&e.name))
        .collect();

    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter().map(|e| e.name.as_str()).collect();
        ace.progress(&format!("Registering missing: {}", names.join(", ")));
        McpRegister { backend, entries: &entries }.run(ace)?;
    }

    // TODO: health check via one-shot backend prompt (PROD9-53)

    if missing.is_empty() {
        ace.done("all MCP servers registered");
    }

    Ok(())
}

/// `ace mcp check` — health check only, no mutations.
fn run_check(ace: &mut Ace) -> Result<(), CmdError> {
    ace.require_state()?;

    let (backend, entries) = load_school_mcp(ace)?;
    if entries.is_empty() {
        ace.hint("no MCP servers defined in school");
        return Ok(());
    }

    let registered = backend.mcp_list();
    let school_names: HashSet<&str> = entries.iter().map(|e| e.name.as_str()).collect();

    for entry in &entries {
        if registered.contains(&entry.name) {
            ace.done(&format!("{}", entry.name));
        } else {
            ace.warn(&format!("{} (not registered)", entry.name));
        }
    }

    // Report registered servers not in school config
    for name in &registered {
        if !school_names.contains(name.as_str()) {
            ace.hint(&format!("{name} (not in school, ignored)"));
        }
    }

    // TODO: one-shot health check via backend prompt (PROD9-53)

    Ok(())
}

/// `ace mcp reset [name]` / `ace mcp clear [name]` — remove servers.
fn run_reset(ace: &mut Ace, name: Option<String>) -> Result<(), CmdError> {
    ace.require_state()?;

    let (backend, entries) = load_school_mcp(ace)?;
    let registered = backend.mcp_list();

    let names: Vec<String> = match name {
        Some(n) => {
            if !registered.contains(&n) {
                ace.warn(&format!("'{n}' is not registered, nothing to remove"));
                return Ok(());
            }
            vec![n]
        }
        None => {
            let school_registered: Vec<String> = entries.iter()
                .map(|e| e.name.clone())
                .filter(|n| registered.contains(n))
                .collect();

            if school_registered.is_empty() {
                ace.hint("no school-defined MCP servers are registered");
                return Ok(());
            }
            school_registered
        }
    };

    McpRemove { backend, names: &names }.run(ace)
        .map_err(|e| CmdError::Other(e))?;

    Ok(())
}

/// Load school MCP entries and backend from current state.
fn load_school_mcp(ace: &Ace) -> Result<(crate::config::backend::Backend, Vec<McpDecl>), CmdError> {
    let backend = ace.state().backend;
    let entries = ace.state().school.as_ref()
        .map(|s| s.mcp.clone())
        .unwrap_or_default();
    Ok((backend, entries))
}
