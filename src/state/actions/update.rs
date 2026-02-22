use std::path::Path;
use std::process::Command;

use crate::config;
use crate::session::Session;
use super::setup::SetupError;

/// Git fetch + reset school cache to latest origin/main.
/// Aborts if cache has uncommitted changes (user should `school propose` or discard first).
pub struct Update<'a> {
    pub specifier: &'a str,
    pub project_dir: &'a Path,
}

impl Update<'_> {
    pub fn run(&self, _session: &mut Session<'_>) -> Result<(), SetupError> {
        let school_paths = config::school_paths::resolve(self.project_dir, self.specifier)?;

        let cache = match &school_paths.cache {
            Some(c) => c,
            None => return Ok(()), // embedded school
        };

        if !cache.join(".git").exists() {
            return Err(SetupError::Clone(format!(
                "school not installed: {}", self.specifier
            )));
        }

        if is_dirty(cache)? {
            return Err(SetupError::Clone(
                "school cache has uncommitted changes, run `ace school propose` or discard first".to_string()
            ));
        }

        git_fetch(cache)?;
        git_reset_to_origin_main(cache)?;

        Ok(())
    }
}

fn is_dirty(repo: &Path) -> Result<bool, SetupError> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo)
        .output()
        .map_err(|e| SetupError::Clone(format!("git status: {e}")))?;

    Ok(!output.stdout.is_empty())
}

fn git_fetch(repo: &Path) -> Result<(), SetupError> {
    let status = Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(repo)
        .status()
        .map_err(|e| SetupError::Clone(format!("git fetch: {e}")))?;

    if !status.success() {
        return Err(SetupError::Clone(format!("git fetch exited {status}")));
    }
    Ok(())
}

fn git_reset_to_origin_main(repo: &Path) -> Result<(), SetupError> {
    let status = Command::new("git")
        .args(["reset", "--hard", "origin/main"])
        .current_dir(repo)
        .status()
        .map_err(|e| SetupError::Clone(format!("git reset: {e}")))?;

    if !status.success() {
        return Err(SetupError::Clone(format!("git reset exited {status}")));
    }
    Ok(())
}
