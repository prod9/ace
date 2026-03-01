use clap::Subcommand;

use crate::ace::Ace;
use crate::state::actions::school_init::SchoolInit;
use crate::state::actions::school_update::{SchoolUpdate, SchoolUpdateResult};
use crate::term_ui::{Tui, Workflow};

use super::CmdError;

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new school repository
    Init {
        /// School display name
        #[arg(long)]
        name: Option<String>,
        /// Overwrite existing school.toml
        #[arg(long)]
        force: bool,
    },
    /// Re-fetch all imported skills from their sources
    Update,
}

pub async fn run(ace: &mut Ace, command: Command) {
    match command {
        Command::Init { name, force } => {
            let result = run_init(ace, name, force);
            super::exit_on_err(ace, result);
        }
        Command::Update => {
            let result = run_update(ace);
            super::exit_on_err(ace, result);
        }
    }
}

fn run_init(ace: &mut Ace, name: Option<String>, force: bool) -> Result<(), CmdError> {
    match name {
        Some(name) => {
            let project_dir = std::env::current_dir()?;
            SchoolInit {
                name: &name,
                project_dir: &project_dir,
                force,
            }
            .run(ace)?;
        }
        None => {
            Tui::new(ace).run(Workflow::SchoolInit { force })?;
        }
    }
    Ok(())
}

fn run_update(ace: &mut Ace) -> Result<(), CmdError> {
    let school_root = resolve_school_root()?;

    let mode = ace.output_mode();
    let mut ace = Ace::new(mode);

    let result = SchoolUpdate { school_root: &school_root }.run(&mut ace)?;
    match result {
        SchoolUpdateResult::NoImports => ace.warn("no imports to update"),
        SchoolUpdateResult::Updated { .. } => {}
    }
    Ok(())
}

fn resolve_school_root() -> Result<std::path::PathBuf, CmdError> {
    let cwd = std::env::current_dir()?;

    if cwd.join("school.toml").exists() {
        return Ok(cwd);
    }

    let ace_toml_path = cwd.join("ace.toml");
    if ace_toml_path.exists() {
        let ace = crate::config::ace_toml::load(&ace_toml_path)?;
        let paths = crate::config::school_paths::resolve(&cwd, &ace.school)?;
        return Ok(paths.root);
    }

    Err(CmdError::NoSchool)
}
