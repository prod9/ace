use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use crate::ace::OutputMode;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git {cmd}: {source}")]
    Exec { cmd: String, source: std::io::Error },
    #[error("git {cmd}: {status}")]
    Exit { cmd: String, status: ExitStatus },
}

pub struct Git<'a> {
    repo: &'a Path,
    mode: OutputMode,
}

impl<'a> Git<'a> {
    pub fn new(repo: &'a Path, mode: OutputMode) -> Self {
        Self { repo, mode }
    }

    pub fn is_dirty(&self) -> Result<bool, GitError> {
        let out = self.output(&["status", "--porcelain"])?;
        Ok(!out.is_empty())
    }

    pub fn fetch_shallow(&self, remote: &str, branch: &str) -> Result<(), GitError> {
        self.run(&["fetch", "--depth", "1", "--no-tags", remote, branch])
    }

    pub fn rev_parse(&self, refspec: &str) -> Result<String, GitError> {
        Ok(self.output(&["rev-parse", refspec])?.trim().to_string())
    }

    pub fn merge_ff_only(&self, target: &str) -> Result<(), GitError> {
        self.run(&["merge", "--ff-only", target])
    }

    pub fn is_ahead_of(&self, remote_ref: &str) -> Result<bool, GitError> {
        let out = self.output(&["rev-list", "--count", &format!("{remote_ref}..HEAD")])?;
        Ok(out.trim() != "0")
    }

    pub fn current_branch(&self) -> Result<String, GitError> {
        Ok(self.output(&["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_string())
    }

    pub fn checkout_branch(&self, branch: &str) -> Result<(), GitError> {
        self.run(&["checkout", branch])
    }

    pub fn diff_name_status(
        &self,
        from: &str,
        to: &str,
        path_filter: Option<&str>,
    ) -> Result<String, GitError> {
        let mut args = vec!["diff", "--name-status", from, to];
        if let Some(filter) = path_filter {
            args.push("--");
            args.push(filter);
        }
        self.output(&args)
    }

    pub fn intent_to_add_all(&self) -> Result<(), GitError> {
        self.run(&["add", "-N", "."])
    }

    pub fn diff(&self) -> Result<String, GitError> {
        let color = match self.mode {
            OutputMode::Human => "--color=always",
            OutputMode::Porcelain | OutputMode::Silent => "--color=never",
        };
        self.output(&["diff", color])
    }

    fn run(&self, args: &[&str]) -> Result<(), GitError> {
        let cmd_str = format!("git {}", args.join(" "));

        let status = Command::new("git")
            .args(args)
            .current_dir(self.repo)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| GitError::Exec { cmd: cmd_str.clone(), source: e })?;

        if !status.success() {
            return Err(GitError::Exit { cmd: cmd_str, status });
        }
        Ok(())
    }

    fn output(&self, args: &[&str]) -> Result<String, GitError> {
        let cmd_str = format!("git {}", args.join(" "));

        let out = Command::new("git")
            .args(args)
            .current_dir(self.repo)
            .output()
            .map_err(|e| GitError::Exec { cmd: cmd_str.clone(), source: e })?;

        if !out.status.success() {
            return Err(GitError::Exit { cmd: cmd_str, status: out.status });
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// Standalone — no repo context needed.
pub fn clone_shallow(url: &str, dest: &Path) -> Result<(), GitError> {
    let cmd_str = format!("git clone --depth 1 --single-branch --no-tags {url}");

    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--single-branch", "--no-tags", url])
        .arg(dest)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| GitError::Exec { cmd: cmd_str.clone(), source: e })?;

    if !status.success() {
        return Err(GitError::Exit { cmd: cmd_str, status });
    }
    Ok(())
}
