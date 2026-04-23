use std::path::Path;

use crate::ace::Ace;
use crate::config::backend::Backend;
use crate::config::school_paths;
use crate::config::ConfigError;

use super::link_skills;
use super::{clone, Link, Pull, PullOutcome, SkillChange};

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
//   codex:    skills ✓  rules ✗  commands ✗  agents ✗
fn is_supported(backend: Backend, folder: &str) -> bool {
    matches!(
        (backend, folder),
        (_, "skills") | (Backend::Claude | Backend::Flaude, _)
    )
}

impl Prepare<'_> {
    pub async fn run(&self, ace: &mut Ace) -> Result<PrepareResult, PrepareError> {
        // Decide install-vs-update by on-disk state, not the index.
        // A stale index entry (clone dir deleted, pre-XDG upgrade, etc.) would
        // otherwise route us into Pull and hit "school not installed".
        let school_paths = school_paths::resolve(self.project_dir, self.specifier)?;
        let needs_clone = school_paths
            .clone_path
            .as_ref()
            .is_some_and(|p| !p.join(".git").exists());

        let (changes, school_is_dirty) = if needs_clone {
            clone::Clone {
                specifier: self.specifier,
                project_dir: self.project_dir,
            }
            .run(ace)
            .await?;
            (Vec::new(), false)
        } else {
            let outcome = (Pull {
                specifier: self.specifier,
                project_dir: self.project_dir,
                force: false,
            })
            .run(ace)?;
            outcome.emit(ace);
            match outcome {
                PullOutcome::Dirty { .. } => (Vec::new(), true),
                PullOutcome::Updated { changes } => (changes, false),
                _ => (Vec::new(), false),
            }
        };

        // Resolve which skills to link before constructing Link.
        // require_state was already invoked transitively via Pull/specifier;
        // tree is available on State.config.
        ace.require_state()?;
        let tree = ace.state().config.clone();
        let prepared = link_skills::prepare(&school_paths.root, &tree)
            .map_err(PrepareError::Write)?;

        let result = Link {
            school_root: &school_paths.root,
            project_dir: self.project_dir,
            backend_dir: self.backend_dir,
            skills: &prepared.desired,
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
        link_skills::emit_warnings(ace, &prepared, &result);

        Ok(PrepareResult {
            changes,
            school_is_dirty,
        })
    }
}
