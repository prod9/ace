use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::config;
use crate::session::Session;
use super::setup::SetupError;

const FETCH_COOLDOWN: Duration = Duration::from_secs(15 * 60);

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

        if is_stale(cache) {
            git_fetch(cache, self.specifier)?;
            git_reset_to_origin_main(cache)?;
        }

        Ok(())
    }
}

/// Check if the last fetch was longer ago than FETCH_COOLDOWN.
/// Returns true (stale) if FETCH_HEAD is missing or old.
fn is_stale(repo: &Path) -> bool {
    let fetch_head = repo.join(".git/FETCH_HEAD");
    let age = fetch_head.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok());

    match age {
        Some(d) => d > FETCH_COOLDOWN,
        None => true,
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

fn git_fetch(repo: &Path, specifier: &str) -> Result<(), SetupError> {
    let sp = crate::status::spinner(&format!("Fetching {specifier} from origin"));
    let status = Command::new("git")
        .args(["fetch", "--depth", "1", "--no-tags", "origin", "main"])
        .current_dir(repo)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| SetupError::Clone(format!("git fetch: {e}")))?;
    sp.finish_and_clear();

    if !status.success() {
        return Err(SetupError::Clone(format!("git fetch exited {status}")));
    }
    crate::status::done(&format!("Fetched {specifier}"));
    Ok(())
}

fn git_reset_to_origin_main(repo: &Path) -> Result<(), SetupError> {
    let status = Command::new("git")
        .args(["reset", "--hard", "origin/main"])
        .current_dir(repo)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| SetupError::Clone(format!("git reset: {e}")))?;

    if !status.success() {
        return Err(SetupError::Clone(format!("git reset exited {status}")));
    }
    Ok(())
}
