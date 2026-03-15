use std::path::Path;

use crate::ace::Ace;
use crate::config;
use crate::config::ConfigError;
use crate::templates;

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
    pub async fn run(&self, ace: &mut Ace) -> Result<(), SetupError> {
        if !super::is_git_repo(self.project_dir) {
            return Err(SetupError::NotInGitRepo);
        }

        let ace_paths = config::paths::resolve(self.project_dir)?;
        if ace_paths.project.exists() {
            return Err(SetupError::AlreadySetUp);
        }

        WriteConfig::project(&ace_paths.project, self.specifier)?;

        // Resolve backend from config layers (user/project/school).
        ace.require_state()?;
        let backend = ace.state().backend;

        Prepare {
            specifier: self.specifier,
            project_dir: self.project_dir,
            skills_dir: backend.skills_dir(),
            backend,
        }
        .run(ace)
        .await?;

        // Reload state after Prepare (school.toml now available).
        ace.reload_state()?;
        let backend = ace.state().backend;

        let instructions = self.project_dir.join(backend.instructions_file());
        if !instructions.exists() {
            let school_name = ace.state().school.as_ref()
                .ok_or(ConfigError::NoSchool)?
                .name.clone();

            let skills_dir = self.project_dir.join(backend.skills_dir());
            let ctx = templates::PromptCtx::new(&skills_dir, &school_name);
            let content = templates::render(templates::PROJECT_CLAUDE_MD, &ctx);

            std::fs::write(&instructions, content)
                .map_err(SetupError::Write)?;
            ace.done(&format!("Created {}", backend.instructions_file()));
        }

        Ok(())
    }
}
