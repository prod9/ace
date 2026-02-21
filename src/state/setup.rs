use std::path::Path;

use crate::config;
use crate::session::Session;

use super::actions;
use super::actions::install::Install;
use super::actions::link::Link;

#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("bad specifier: {0}")]
    InvalidSpecifier(#[from] config::school_paths::ResolveError),
    #[error("{0}")]
    SchoolConfig(#[from] config::ConfigError),
    #[error("{0}")]
    Path(#[from] config::paths::PathError),
    #[error("not in git repo, git init?")]
    NotInGitRepo,
    #[error("no cached schools, ace setup <owner/repo>?")]
    NoCachedSchools,
    #[error("clone failed: {0}")]
    Clone(String),
    #[error("write failed: {0}")]
    WriteConfig(std::io::Error),
}

pub struct Setup<'a> {
    pub specifier: Option<&'a str>,
    pub project_dir: &'a Path,
}

impl Setup<'_> {
    pub async fn run(&self, session: &mut Session<'_>) -> Result<(), SetupError> {
        if !actions::is_git_repo(self.project_dir) {
            return Err(SetupError::NotInGitRepo);
        }

        match self.specifier {
            Some(specifier) => {
                Install { project_dir: self.project_dir, specifier }.run(session).await
            }
            None => {
                Link { project_dir: self.project_dir }.run(session).await
            }
        }
    }
}
