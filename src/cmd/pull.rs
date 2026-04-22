use crate::ace::Ace;
use crate::config::school_paths;
use crate::config::ConfigError;
use crate::actions::project::{Link, Pull};

use super::CmdError;

pub fn run(ace: &mut Ace) {
    let result = run_inner(ace);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    ace.require_state()?;

    let specifier = ace.state().school_specifier.clone()
        .ok_or(ConfigError::NoSchool)?;

    let project_dir = ace.project_dir().to_path_buf();

    let outcome = (Pull {
        specifier: &specifier,
        project_dir: &project_dir,
        force: true,
    })
    .run(ace)?;
    outcome.emit(ace);

    // Re-link in case new folders appeared.
    let school_paths = school_paths::resolve(&project_dir, &specifier)?;
    let backend = ace.state().backend;

    Link {
        school_root: &school_paths.root,
        project_dir: &project_dir,
        backend_dir: backend.backend_dir(),
    }
    .run(ace)?;

    Ok(())
}
