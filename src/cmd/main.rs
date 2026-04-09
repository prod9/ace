use crate::ace::Ace;
use crate::config::ConfigError;
use crate::config::backend::SessionOpts;
use crate::state::actions::register_mcp::McpRegister;
use crate::state::actions::prepare_school::{Prepare, PrepareResult};
use crate::templates::session::build_session_prompt;

use super::CmdError;

pub async fn run(ace: &mut Ace, backend_args: Vec<String>, should_resume: bool) {
    let result = run_inner(ace, backend_args, should_resume).await;
    super::exit_on_err(ace, result);
}

async fn run_inner(ace: &mut Ace, backend_args: Vec<String>, should_resume: bool) -> Result<(), CmdError> {
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
        match backend.supports_trust(trust) {
            Ok(()) => {
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

    let resume = should_resume && ace.state().resume;
    if resume {
        ace.hint("Resuming previous session. If this fails, run: ace new");
    }

    ace.separator();

    backend.exec_session(SessionOpts {
        trust,
        session_prompt,
        project_dir,
        env: ace.state().env.clone(),
        extra_args: backend_args,
        resume,
    })?;

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
