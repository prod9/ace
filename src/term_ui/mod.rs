use crate::ace::Ace;
use crate::state::actions::school_init::{SchoolInit, SchoolInitError};

pub const LOGO: &str = r"
░█▀█░█▀▀░█▀▀
░█▀█░█░░░█▀▀
░▀░▀░▀▀▀░▀▀▀";

#[derive(Debug, thiserror::Error)]
pub enum TermError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    SchoolInit(#[from] crate::state::actions::school_init::SchoolInitError),
    #[error("cancelled")]
    Cancelled,
}

pub enum Workflow {
    SchoolInit,
}

pub struct Tui<'a> {
    ace: &'a mut Ace,
}

impl<'a> Tui<'a> {
    pub fn new(ace: &'a mut Ace) -> Self {
        Self { ace }
    }

    pub fn run(&mut self, workflow: Workflow) -> Result<(), TermError> {
        match workflow {
            Workflow::SchoolInit => self.school_init(),
        }
    }

    fn school_init(&mut self) -> Result<(), TermError> {
        let project_dir = std::env::current_dir()?;

        if !crate::state::actions::is_git_repo(&project_dir) {
            return Err(SchoolInitError::NotInGitRepo.into());
        }
        if project_dir.join("school.toml").exists() {
            return Err(SchoolInitError::AlreadyExists.into());
        }

        println!("{LOGO}\n");

        let name = inquire::Text::new("School name:")
            .prompt()
            .map_err(map_inquire_err)?;

        let mut session = self.ace.session();
        SchoolInit { name: &name, project_dir: &project_dir }.run(&mut session)?;

        println!("Created {}", project_dir.join("school.toml").display());
        Ok(())
    }
}

/// Prompt user to select from a list. Returns the selected item.
pub fn select(prompt: &str, options: Vec<String>) -> Result<String, TermError> {
    inquire::Select::new(prompt, options)
        .prompt()
        .map_err(map_inquire_err)
}

fn map_inquire_err(e: inquire::InquireError) -> TermError {
    match e {
        inquire::InquireError::OperationCanceled
        | inquire::InquireError::OperationInterrupted => TermError::Cancelled,
        other => TermError::Io(std::io::Error::other(other)),
    }
}
