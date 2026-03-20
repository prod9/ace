use std::path::Path;

use crate::ace::Ace;
use crate::config;
use crate::config::backend::Backend;
use crate::config::ConfigError;
use crate::templates;

use super::gitignore::UpdateGitignore;
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

        let backend = Backend::default();
        Prepare {
            specifier: self.specifier,
            project_dir: self.project_dir,
            backend_dir: backend.backend_dir(),
            backend,
        }
        .run(ace)
        .await?;

        UpdateGitignore {
            project_dir: self.project_dir,
            backend_dir: backend.backend_dir(),
        }
        .run(ace)
        .map_err(SetupError::Write)?;

        let instructions = self.project_dir.join(backend.instructions_file());
        if !instructions.exists() {
            ace.require_state()?;
            let school_name = ace.state().school.as_ref()
                .ok_or(ConfigError::NoSchool)?
                .name.clone();

            let backend_dir_name = backend.backend_dir();
            let tpl = templates::Template::parse(templates::builtins::PROJECT_CLAUDE_MD);
            let content = tpl.substitute(&std::collections::HashMap::from([
                ("school_name".to_string(), school_name),
                ("backend_dir".to_string(), backend_dir_name.to_string()),
            ]));

            std::fs::write(&instructions, content)
                .map_err(SetupError::Write)?;
            ace.done(&format!("Created {}", backend.instructions_file()));
        }

        Ok(())
    }
}
