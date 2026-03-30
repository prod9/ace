use std::path::Path;

use crate::ace::Ace;
use crate::config::backend::Backend;
use crate::config::index_toml;
use crate::config::school_paths;
use crate::config::ConfigError;

use super::install::Install;
use super::link::Link;
use super::update::{SkillChange, Update, UpdateResult};

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
    pub backend_dir: &'a str,
    pub backend: Backend,
}

#[derive(Debug, Default)]
pub struct PrepareResult {
    pub changes: Vec<SkillChange>,
    pub school_is_dirty: bool,
}

// Backend support matrix — which folders each backend natively supports.
//   claude:   skills ✓  rules ✓  commands ✓  agents ✓
//   opencode: skills ✓  rules ✗  commands ✓  agents ✓
//   codex:    skills ✓  rules ✗  commands ✗  agents ✗
fn is_supported(backend: Backend, folder: &str) -> bool {
    match (backend, folder) {
        (_, "skills") => true,
        (Backend::Claude | Backend::Flaude, _) => true,
        (Backend::OpenCode, "commands" | "agents") => true,
        _ => false,
    }
}

impl Prepare<'_> {
    pub async fn run(&self, ace: &mut Ace) -> Result<PrepareResult, PrepareError> {
        let update_result = if is_cached(self.specifier)? {
            (Update {
                specifier: self.specifier,
                project_dir: self.project_dir,
            })
            .run(ace)?
        } else {
            Install {
                specifier: self.specifier,
                project_dir: self.project_dir,
            }
            .run(ace)
            .await?;
            UpdateResult::default()
        };

        let school_paths = school_paths::resolve(self.project_dir, self.specifier)?;

        let result = Link {
            school_root: &school_paths.root,
            project_dir: self.project_dir,
            backend_dir: self.backend_dir,
        }
        .run(ace)?;
        for folder in &result.folders {
            if folder.adopted {
                ace.done(&format!("Moved previous {0} to previous-{0}/", folder.name));
            }
            if folder.linked {
                if is_supported(self.backend, folder.name) {
                    ace.done(&format!("Linked {}", folder.name));
                } else {
                    ace.warn(&format!(
                        "Linked {0}/ — not natively supported by {1} (linked for future compatibility)",
                        folder.name,
                        self.backend.binary(),
                    ));
                }
            }
        }

        Ok(PrepareResult {
            changes: update_result.changes,
            school_is_dirty: update_result.school_is_dirty,
        })
    }
}

fn is_cached(specifier: &str) -> Result<bool, PrepareError> {
    let index_path = index_toml::index_path()
        .map_err(|e| PrepareError::Clone(format!("index path: {e}")))?;
    let index = index_toml::load(&index_path)
        .map_err(|e| PrepareError::Clone(format!("load index: {e}")))?;
    Ok(index.school.iter().any(|s| s.specifier == specifier))
}
