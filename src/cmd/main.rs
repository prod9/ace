use crate::ace::Ace;
use crate::config::ConfigError;
use crate::state::actions::exec::Exec;
use crate::state::actions::mcp_register::McpRegister;
use crate::state::actions::prepare::{Prepare, PrepareResult};
use crate::templates::session::build_session_prompt;

use super::CmdError;

pub async fn run(ace: &mut Ace, backend_args: Vec<String>) {
    let result = run_inner(ace, backend_args).await;
    super::exit_on_err(ace, result);
}

async fn run_inner(ace: &mut Ace, mut backend_args: Vec<String>) -> Result<(), CmdError> {
    ace.require_state()?;

    let specifier = ace.state().school_specifier.clone()
        .ok_or(ConfigError::NoSchool)?;

    let prepare_result = prepare_school(ace, &specifier).await?;

    let project_dir = ace.project_dir().to_path_buf();
    let school_paths = ace.require_school()?;
    let school_cache = school_paths.cache.clone();

    let school = ace.state().school.as_ref()
        .ok_or(ConfigError::NoSchool)?;

    let backend_dir = project_dir.join(ace.state().backend.backend_dir());
    let session_prompt = build_session_prompt(
        &school.name,
        &school.session_prompt,
        &ace.state().session_prompt,
        &backend_dir,
        &prepare_result.changes,
        school_cache.as_deref(),
        prepare_result.school_is_dirty,
    );

    let backend = ace.state().backend;
    let trust = ace.state().trust;
    if !trust.is_default() {
        match backend.trust_args(trust) {
            Ok(args) => {
                backend_args.extend(args);
                let label = match trust {
                    crate::config::ace_toml::Trust::Auto => "auto mode — AI decides approvals",
                    crate::config::ace_toml::Trust::Yolo => "yolo mode — permission prompts disabled",
                    _ => "trust mode active",
                };
                ace.warn(label);
            }
            Err(msg) => ace.warn(&format!("trust ignored: {msg}")),
        }
    }

    ace.separator();

    Exec {
        backend,
        session_prompt,
        project_dir,
        env: ace.state().env.clone(),
        backend_args,
    }
    .run(ace)?;

    Ok(())
}

/// Shared workflow: prepare school (install/update/link) → register MCP servers.
///
/// Called by both bare `ace` and `ace setup`. Reloads state after linking so
/// school.toml is available for MCP registration and downstream callers.
pub(super) async fn prepare_school(
    ace: &mut Ace,
    specifier: &str,
) -> Result<PrepareResult, CmdError> {
    let preliminary_backend = ace.state().backend;
    let project_dir = ace.project_dir().to_path_buf();

    let prepare_result = (Prepare {
        specifier,
        project_dir: &project_dir,
        backend_dir: preliminary_backend.backend_dir(),
        backend: preliminary_backend,
    })
    .run(ace)
    .await?;

    // Reload with fresh school.toml after Prepare.
    ace.reload_state()?;

    // Register MCP servers from school.toml.
    let mcp_entries: Vec<_> = ace.state().school.as_ref()
        .map(|s| s.mcp.clone())
        .unwrap_or_default();

    if mcp_entries.is_empty() {
        return Ok(prepare_result);
    }

    let backend = ace.state().backend;
    let registered = backend.mcp_list();
    let pending: Vec<&str> = mcp_entries.iter()
        .filter(|e| !registered.contains(&e.name))
        .map(|e| e.name.as_str())
        .collect();

    if pending.is_empty() {
        return Ok(prepare_result);
    }

    let prompt = format!("Register MCP server(s): {}?", pending.join(", "));
    if !ace.prompt_confirm(&prompt, true)? {
        return Ok(prepare_result);
    }

    if let Err(e) = (McpRegister { backend, entries: &mcp_entries }).run(ace) {
        ace.warn(&format!("MCP registration failed: {e}"));
    }

    Ok(prepare_result)
}
