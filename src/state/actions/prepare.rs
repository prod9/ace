use std::path::Path;

use crate::config::index_toml;
use crate::session::Session;

use super::install::Install;
use super::link::Link;
use super::setup::SetupError;
use super::update::Update;

/// Ensure school is ready: install if not cached, update if cached, link into project.
pub struct Prepare<'a> {
    pub specifier: &'a str,
    pub project_dir: &'a Path,
    pub skills_dir: &'a str,
}

impl Prepare<'_> {
    pub async fn run(&self, session: &mut Session<'_>) -> Result<(), SetupError> {
        if is_cached(self.specifier)? {
            Update {
                specifier: self.specifier,
                project_dir: self.project_dir,
            }
            .run(session)?;
        } else {
            Install {
                specifier: self.specifier,
                project_dir: self.project_dir,
            }
            .run(session)
            .await?;
        }

        let result = Link {
            specifier: self.specifier,
            project_dir: self.project_dir,
            skills_dir: self.skills_dir,
        }
        .run(session)?;

        if result.linked > 0 {
            eprintln!("Linked {} skills", result.linked);
        }

        Ok(())
    }
}

fn is_cached(specifier: &str) -> Result<bool, SetupError> {
    let index_path = index_toml::index_path()
        .map_err(|e| SetupError::Clone(format!("index path: {e}")))?;
    let index = index_toml::load(&index_path)
        .map_err(|e| SetupError::Clone(format!("load index: {e}")))?;
    Ok(index.school.iter().any(|s| s.specifier == specifier))
}
