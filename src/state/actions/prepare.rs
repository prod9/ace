use std::path::Path;

use crate::ace::Ace;
use crate::config::index_toml;
use crate::config::school_paths;
use crate::config::ConfigError;

use super::install::Install;
use super::link::Link;
use super::update::{SkillChange, Update};

#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("clone failed: {0}")]
    Clone(String),
    #[error("write failed: {0}")]
    Write(std::io::Error),
}

/// Ensure school is ready: install if not cached, update if cached, link into project.
pub struct Prepare<'a> {
    pub specifier: &'a str,
    pub project_dir: &'a Path,
    pub skills_dir: &'a str,
}

#[derive(Debug, Default)]
pub struct PrepareResult {
    pub changes: Vec<SkillChange>,
}

impl Prepare<'_> {
    pub async fn run(&self, ace: &mut Ace) -> Result<PrepareResult, PrepareError> {
        let changes = if is_cached(self.specifier)? {
            (Update {
                specifier: self.specifier,
                project_dir: self.project_dir,
            })
            .run(ace)?
            .changes
        } else {
            Install {
                specifier: self.specifier,
                project_dir: self.project_dir,
            }
            .run(ace)
            .await?;
            Vec::new()
        };

        let school_paths = school_paths::resolve(self.project_dir, self.specifier)?;

        let result = Link {
            school_root: &school_paths.root,
            project_dir: self.project_dir,
            skills_dir: self.skills_dir,
        }
        .run(ace)?;

        if result.skills_adopted {
            ace.done("Moved previous skills to previous-skills/");
        }
        if result.rules_adopted {
            ace.done("Moved previous rules to previous-rules/");
        }
        if result.skills_linked {
            ace.done("Linked skills");
        }
        if result.rules_linked {
            ace.done("Linked rules");
        }

        Ok(PrepareResult { changes })
    }
}

fn is_cached(specifier: &str) -> Result<bool, PrepareError> {
    let index_path = index_toml::index_path()
        .map_err(|e| PrepareError::Clone(format!("index path: {e}")))?;
    let index = index_toml::load(&index_path)
        .map_err(|e| PrepareError::Clone(format!("load index: {e}")))?;
    Ok(index.school.iter().any(|s| s.specifier == specifier))
}
