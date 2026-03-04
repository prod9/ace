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
            let project_dir = ace.project_dir().to_path_buf();
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
    let school_root = ace.require_school()?.root.clone();

    let result = SchoolUpdate { school_root: &school_root }.run(ace)?;
    match result {
        SchoolUpdateResult::NoImports => ace.warn("no imports to update"),
        SchoolUpdateResult::Updated { .. } => {}
    }
    Ok(())
}
