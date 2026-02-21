use std::path::Path;
use std::process::Command;

use crate::session::Session;

#[derive(Debug, thiserror::Error)]
pub enum ProposeError {
    #[error("no school linked, run ace setup first")]
    NoSchool,
    #[error("school cache not found at {0}")]
    NoCacheDir(String),
    #[error("no changes to propose")]
    NoChanges,
    #[error("git: {0}")]
    Git(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] crate::config::school_paths::ResolveError),
}

/// Propose local school modifications back to the upstream school repo.
///
/// Creates a branch in the school's local clone, commits any changes to
/// skills/conventions, pushes the branch, and opens a PR via `gh`.
pub struct Propose<'a> {
    pub project_dir: &'a Path,
}

impl Propose<'_> {
    pub fn run(&self, session: &mut Session<'_>) -> Result<String, ProposeError> {
        let specifier = session
            .state
            .school_specifier
            .as_deref()
            .ok_or(ProposeError::NoSchool)?;

        let school_paths = crate::config::school_paths::resolve(self.project_dir, specifier)?;
        let cache = school_paths
            .cache
            .as_deref()
            .ok_or_else(|| ProposeError::NoCacheDir("embedded school".to_string()))?;

        if !cache.join(".git").exists() {
            return Err(ProposeError::NoCacheDir(cache.display().to_string()));
        }

        // Check for uncommitted changes
        if !has_changes(cache)? {
            return Err(ProposeError::NoChanges);
        }

        let branch = format!("ace/propose-{}", timestamp());
        create_branch(cache, &branch)?;
        stage_and_commit(cache)?;
        push_branch(cache, &branch)?;

        let pr_url = open_pr(cache, &branch)?;
        Ok(pr_url)
    }
}

fn has_changes(repo: &Path) -> Result<bool, ProposeError> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo)
        .output()
        .map_err(|e| ProposeError::Git(format!("git status: {e}")))?;

    Ok(!output.stdout.is_empty())
}

fn create_branch(repo: &Path, branch: &str) -> Result<(), ProposeError> {
    let status = Command::new("git")
        .args(["checkout", "-b", branch])
        .current_dir(repo)
        .status()
        .map_err(|e| ProposeError::Git(format!("git checkout: {e}")))?;

    if !status.success() {
        return Err(ProposeError::Git(format!(
            "git checkout -b {branch} exited {status}"
        )));
    }
    Ok(())
}

fn stage_and_commit(repo: &Path) -> Result<(), ProposeError> {
    let status = Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo)
        .status()
        .map_err(|e| ProposeError::Git(format!("git add: {e}")))?;

    if !status.success() {
        return Err(ProposeError::Git(format!("git add exited {status}")));
    }

    let status = Command::new("git")
        .args(["commit", "-m", "Propose school changes from ace"])
        .current_dir(repo)
        .status()
        .map_err(|e| ProposeError::Git(format!("git commit: {e}")))?;

    if !status.success() {
        return Err(ProposeError::Git(format!("git commit exited {status}")));
    }
    Ok(())
}

fn push_branch(repo: &Path, branch: &str) -> Result<(), ProposeError> {
    let status = Command::new("git")
        .args(["push", "-u", "origin", branch])
        .current_dir(repo)
        .status()
        .map_err(|e| ProposeError::Git(format!("git push: {e}")))?;

    if !status.success() {
        return Err(ProposeError::Git(format!("git push exited {status}")));
    }
    Ok(())
}

fn open_pr(repo: &Path, branch: &str) -> Result<String, ProposeError> {
    let output = Command::new("gh")
        .args([
            "pr",
            "create",
            "--title",
            "Propose school changes",
            "--body",
            "Changes proposed via `ace school propose`.",
            "--head",
            branch,
        ])
        .current_dir(repo)
        .output()
        .map_err(|e| ProposeError::Git(format!("gh pr create: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ProposeError::Git(format!("gh pr create failed: {stderr}")));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(url)
}

fn timestamp() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}
