use std::process::Command;

use crate::config::school_paths::SchoolPaths;
use crate::session::Session;
use super::setup::SetupError;

pub struct DownloadSchool<'a> {
    pub paths: &'a SchoolPaths,
}

impl DownloadSchool<'_> {
    pub fn run(&self, _session: &mut Session<'_>) -> Result<(), SetupError> {
        let cache = match &self.paths.cache {
            Some(c) => c,
            None => return Ok(()), // embedded school, nothing to download
        };

        if cache.join(".git").exists() {
            self.pull(cache)
        } else {
            self.clone(cache)
        }
    }

    fn clone(&self, cache: &std::path::Path) -> Result<(), SetupError> {
        if let Some(parent) = cache.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SetupError::Clone(format!("mkdir: {e}")))?;
        }

        let repo = self.paths.source.split_once(':').map_or(
            self.paths.source.as_str(),
            |(owner_repo, _)| owner_repo,
        );
        let url = format!("https://github.com/{repo}.git");
        let status = Command::new("git")
            .args(["clone", "--depth", "1", &url])
            .arg(cache)
            .status()
            .map_err(|e| SetupError::Clone(format!("git clone: {e}")))?;

        if !status.success() {
            return Err(SetupError::Clone(format!("git clone exited {status}")));
        }
        Ok(())
    }

    fn pull(&self, cache: &std::path::Path) -> Result<(), SetupError> {
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
