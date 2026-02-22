use std::path::Path;

use crate::config;
use crate::session::Session;

use super::prepare::Prepare;
use super::write_config::WriteConfig;

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
    #[error("already set up, use `ace` to run")]
    AlreadySetUp,
    #[error("clone failed: {0}")]
    Clone(String),
    #[error("write failed: {0}")]
    WriteConfig(std::io::Error),
}

pub struct Setup<'a> {
    pub specifier: &'a str,
    pub project_dir: &'a Path,
}

impl Setup<'_> {
    pub async fn run(&self, session: &mut Session<'_>) -> Result<(), SetupError> {
        if !super::is_git_repo(self.project_dir) {
            return Err(SetupError::NotInGitRepo);
        }

        let ace_paths = config::paths::resolve(self.project_dir)?;
        if ace_paths.project.exists() {
            return Err(SetupError::AlreadySetUp);
        }

        WriteConfig::project(&ace_paths.project, self.specifier)?;

        Prepare {
            specifier: self.specifier,
            project_dir: self.project_dir,
        }
        .run(session)
        .await?;

        Ok(())
    }
}
