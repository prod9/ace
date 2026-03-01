pub mod sink;

use std::path::PathBuf;

use crate::ace::Ace;
use crate::config::school_toml::ServiceDecl;
use crate::events::OutputMode;
use crate::state::actions::add_service::AddService;
use crate::state::actions::school_init::{SchoolInit, SchoolInitError};

#[allow(dead_code)]
pub const LOGO: &str = r"
░█▀█░█▀▀░█▀▀
░█▀█░█░░░█▀▀
░▀░▀░▀▀▀░▀▀▀";

pub const LOGO_COLOR: &str = "\x1b[36m
░█▀█░█▀▀░█▀▀
░█▀█░█░░░█▀▀
░▀░▀░▀▀▀░▀▀▀\x1b[0m";

pub fn logo(mode: OutputMode) -> &'static str {
    match mode {
        OutputMode::Human => LOGO_COLOR,
        _ => "",
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TermError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    SchoolInit(#[from] crate::state::actions::school_init::SchoolInitError),
    #[error("{0}")]
    AddService(#[from] crate::state::actions::add_service::AddServiceError),
    #[error("cancelled")]
    Cancelled,
}

pub enum Workflow {
    SchoolInit { force: bool },
    AddService { school_root: PathBuf },
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
            Workflow::AddService { school_root } => self.add_service(school_root),
        }
    }

    fn school_init(&mut self, force: bool) -> Result<(), TermError> {
        let project_dir = self.ace.project_dir().to_path_buf();
        let toml_path = project_dir.join("school.toml");

        if !crate::state::actions::is_git_repo(&project_dir) {
            return Err(SchoolInitError::NotInGitRepo.into());
        }
        if !force && toml_path.exists() {
            return Err(SchoolInitError::AlreadyExists.into());
        }

        let existing_name = if force && toml_path.exists() {
            crate::config::school_toml::load(&toml_path).ok()
                .map(|s| s.name)
                .filter(|n| !n.is_empty())
        } else {
            None
        };

        let mut prompt = inquire::Text::new("School name:");
        if let Some(ref name) = existing_name {
            prompt = prompt.with_initial_value(name);
        }
        let name = prompt.prompt().map_err(map_inquire_err)?;

        SchoolInit { name: &name, project_dir: &project_dir, force }.run(self.ace)?;

        self.ace.done(&format!("Created {}", toml_path.display()));
        Ok(())
    }

    fn add_service(&mut self, school_root: PathBuf) -> Result<(), TermError> {
        let name = inquire::Text::new("Service name:")
            .with_placeholder("github")
            .prompt()
            .map_err(map_inquire_err)?;

        let authorize_url = inquire::Text::new("Authorize URL:")
            .prompt()
            .map_err(map_inquire_err)?;

        let token_url = inquire::Text::new("Token URL:")
            .prompt()
            .map_err(map_inquire_err)?;

        let client_id = inquire::Text::new("Client ID:")
            .prompt()
            .map_err(map_inquire_err)?;

        let scopes_str = inquire::Text::new("Scopes (comma-separated):")
            .prompt()
            .map_err(map_inquire_err)?;

        let scopes: Vec<String> = scopes_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let service = ServiceDecl { name, authorize_url, token_url, client_id, scopes };
        AddService { school_root: &school_root, service }.run(self.ace)?;
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
