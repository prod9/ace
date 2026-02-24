use std::path::Path;

use crate::config::index_toml;
use crate::config::ConfigError;
use crate::session::Session;

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
    pub async fn run(&self, session: &mut Session<'_>) -> Result<PrepareResult, PrepareError> {
        let changes = if is_cached(self.specifier)? {
            let result = Update {
                specifier: self.specifier,
                project_dir: self.project_dir,
            }
            .run(session)?;
            result.changes
        } else {
            Install {
                specifier: self.specifier,
                project_dir: self.project_dir,
            }
            .run(session)
            .await?;
            Vec::new() // skip on first install
        };

        let result = Link {
            specifier: self.specifier,
            project_dir: self.project_dir,
            skills_dir: self.skills_dir,
        }
        .run(session)?;

        if !result.moved.is_empty() {
            eprintln!(
                "Moved {} previous skill(s) to previous-skills/: {}",
                result.moved.len(),
                result.moved.join(", ")
            );
        }

        if result.linked > 0 {
            eprintln!("Linked {} skills", result.linked);
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
