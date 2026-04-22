use crate::ace::Ace;
use crate::actions::project::link_skills;
use crate::actions::project::{clone, Link, Pull};
use crate::config::school_paths;
use crate::config::ConfigError;

use super::CmdError;

pub async fn run(ace: &mut Ace) {
    let result = run_inner(ace).await;
    super::exit_on_err(ace, result);
}

async fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    ace.require_state()?;

    let specifier = ace.state().school_specifier.clone()
        .ok_or(ConfigError::NoSchool)?;

    let project_dir = ace.project_dir().to_path_buf();
    let school_paths = school_paths::resolve(&project_dir, &specifier)?;

    // Self-heal: if the clone dir is gone (stale index, deleted cache, etc.),
    // clone instead of pulling — Pull would otherwise error "school not installed".
    let needs_clone = school_paths
        .clone_path
        .as_ref()
        .is_some_and(|p| !p.join(".git").exists());

    if needs_clone {
        clone::Clone {
            specifier: &specifier,
            project_dir: &project_dir,
        }
        .run(ace)
        .await?;
    } else {
        let outcome = (Pull {
            specifier: &specifier,
            project_dir: &project_dir,
            force: true,
        })
        .run(ace)?;
        outcome.emit(ace);
    }

    // Re-link in case new folders appeared.
    let backend = ace.state().backend;
    let tree = ace.state().config.clone();
    let prepared = link_skills::prepare(&school_paths.root, &tree)
        .map_err(|e| CmdError::Other(format!("scan school skills: {e}")))?;

    let result = Link {
        school_root: &school_paths.root,
        project_dir: &project_dir,
        backend_dir: backend.backend_dir(),
        skills: &prepared.desired,
    }
    .run(ace)?;
    for warning in &result.skill_warnings {
        ace.warn(warning);
    }
    for unknown in &prepared.resolution.unknown_patterns {
        ace.warn(&format!(
            "skill pattern matched no skill: {} (in {:?} {:?})",
            unknown.pattern, unknown.scope, unknown.field
        ));
    }
    for collision in &prepared.resolution.collisions {
        ace.warn(&format!(
            "skill {} appears in both include_skills and exclude_skills at {:?} scope",
            collision.skill, collision.scope
        ));
    }

    Ok(())
}
