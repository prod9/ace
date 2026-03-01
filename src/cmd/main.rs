use crate::ace::Ace;
use crate::config::ConfigError;
use crate::state::actions::exec::Exec;
use crate::state::actions::prepare::{Prepare, PrepareError, PrepareResult};
use crate::prompts::session::build_session_prompt;

use super::CmdError;

pub async fn run(ace: &mut Ace, backend_args: Vec<String>) {
    let result = run_inner(ace, backend_args).await;
    super::exit_on_err(ace, result);
}

async fn run_inner(ace: &mut Ace, backend_args: Vec<String>) -> Result<(), CmdError> {
    // Pass 1: load tree to get specifier for Prepare.
    ace.require_state()?;

    let specifier = ace.state().school_specifier.clone()
        .ok_or(ConfigError::NoSchool)?;

    // Prepare (install/update/link) needs a preliminary backend for skills_dir.
    let preliminary_backend = ace.state().backend;
    let project_dir = ace.project_dir().to_path_buf();

    let prepare_result = match (Prepare {
        specifier: &specifier,
        project_dir: &project_dir,
        skills_dir: preliminary_backend.skills_dir(),
    })
    .run(ace)
    .await
    {
        Ok(r) => r,
        Err(PrepareError::DirtyCache) => {
            ace.warn(
                "school cache has uncommitted changes, skipping update \
                 (propose changes when ready)",
            );
            PrepareResult::default()
        }
        Err(e) => return Err(e.into()),
    };

    // Pass 2: reload with fresh school.toml after Prepare.
    ace.reload_state()?;

    let school_paths = ace.require_school()?;
    let school_cache = school_paths.cache.clone();

    let school = ace.state().school.as_ref()
        .ok_or(ConfigError::NoSchool)?;

    let skills_dir = project_dir.join(ace.state().backend.skills_dir());
    let session_prompt = build_session_prompt(
        &school.name,
        &school.session_prompt,
        &ace.state().session_prompt,
        &skills_dir,
        &prepare_result.changes,
        school_cache.as_deref(),
    );

    Exec {
        backend: ace.state().backend,
        session_prompt,
        project_dir,
        env: ace.state().env.clone(),
        backend_args,
    }
    .run(ace)?;

    Ok(())
}
