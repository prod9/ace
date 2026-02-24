use std::path::Path;

use crate::config;
use crate::config::backend::Backend;
use crate::config::ConfigError;
use crate::prompts;
use crate::session::Session;

use super::prepare::{Prepare, PrepareError};
use super::write_config::WriteConfig;

#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0}")]
    Prepare(#[from] PrepareError),
    #[error("not in git repo, git init?")]
    NotInGitRepo,
    #[error("already set up, use `ace` to run")]
    AlreadySetUp,
    #[error("write failed: {0}")]
    Write(std::io::Error),
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

        let backend = Backend::default();
        Prepare {
            specifier: self.specifier,
            project_dir: self.project_dir,
            skills_dir: backend.skills_dir(),
        }
        .run(session)
        .await?;

        let instructions = self.project_dir.join(backend.instructions_file());
        if !instructions.exists() {
            let school_paths =
                config::school_paths::resolve(self.project_dir, self.specifier)?;
            let school_toml_path = school_paths.root.join("school.toml");
            let school_toml = config::school_toml::load(&school_toml_path)?;

            let skills_dir = self.project_dir.join(backend.skills_dir());
            let ctx = prompts::PromptCtx::new(&skills_dir, &school_toml.school.name);
            let content = prompts::render(prompts::PROJECT_CLAUDE_MD, &ctx);

            std::fs::write(&instructions, content)
                .map_err(SetupError::Write)?;
            eprintln!("Created {}", backend.instructions_file());
        }

        Ok(())
    }
}
