use std::path::Path;
use std::process::Command;

use crate::config;
use crate::session::Session;
use super::setup::SetupError;

/// Git pull the school cache to get latest content.
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

        let status = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(cache)
            .status()
            .map_err(|e| SetupError::Clone(format!("git pull: {e}")))?;

        if !status.success() {
            return Err(SetupError::Clone(format!("git pull exited {status}")));
        }

        Ok(())
    }
}
