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
    SchoolInit { force: bool },
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
            Workflow::SchoolInit { force } => self.school_init(force),
        }
    }

    fn school_init(&mut self, force: bool) -> Result<(), TermError> {
        let project_dir = std::env::current_dir()?;
        let toml_path = project_dir.join("school.toml");

        if !crate::state::actions::is_git_repo(&project_dir) {
            return Err(SchoolInitError::NotInGitRepo.into());
        }
        if !force && toml_path.exists() {
            return Err(SchoolInitError::AlreadyExists.into());
        }

        let existing_name = if force && toml_path.exists() {
            crate::config::school_toml::load(&toml_path).ok()
                .map(|s| s.school.name)
                .filter(|n| !n.is_empty())
        } else {
            None
        };

        let mut prompt = inquire::Text::new("School name:");
        if let Some(ref name) = existing_name {
            prompt = prompt.with_initial_value(name);
        }
        let name = prompt.prompt().map_err(map_inquire_err)?;

        let mut session = self.ace.session();
        SchoolInit { name: &name, project_dir: &project_dir, force }.run(&mut session)?;

        println!("Created {}", toml_path.display());
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
