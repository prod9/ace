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

    /// Fetch from a remote without using shallow options.
    pub fn fetch(&self, remote: &str, branch: &str) -> Result<(), GitError> {
        self.run(&["fetch", "--no-tags", remote, branch])
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
        Ok(self
            .output(&["rev-parse", "--abbrev-ref", "HEAD"])?
            .trim()
            .to_string())
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
        let cmd_str = args.join(" ");

        let status = Command::new("git")
            .args(args)
            .current_dir(self.repo)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| GitError::Exec {
                cmd: cmd_str.clone(),
                source: e,
            })?;

        if !status.success() {
            return Err(GitError::Exit {
                cmd: cmd_str,
                status,
            });
        }
        Ok(())
    }

    fn output(&self, args: &[&str]) -> Result<String, GitError> {
        let cmd_str = args.join(" ");

        let out = Command::new("git")
            .args(args)
            .current_dir(self.repo)
            .output()
            .map_err(|e| GitError::Exec {
                cmd: cmd_str.clone(),
                source: e,
            })?;

        if !out.status.success() {
            return Err(GitError::Exit {
                cmd: cmd_str,
                status: out.status,
            });
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// Clone a GitHub repo by `owner/repo` specifier into `dest` using a full clone.
pub fn clone_github(source: &str, dest: &Path) -> Result<(), GitError> {
    let url = format!("https://github.com/{source}.git");
    clone_repo(&url, dest)
}

/// Standalone — no repo context needed.
/// Performs a full clone (no `--depth`).
pub fn clone_repo(url: &str, dest: &Path) -> Result<(), GitError> {
    let cmd_str = format!("clone --no-tags {url}");

    let status = Command::new("git")
        .args(["clone", "--no-tags", url])
        .arg(dest)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| GitError::Exec {
            cmd: cmd_str.clone(),
            source: e,
        })?;

    if !status.success() {
        return Err(GitError::Exit {
            cmd: cmd_str,
            status,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn clone_repo_full_history() {
        // Remote repo with two commits
        let remote = TempDir::new().expect("remote tempdir");
        let remote_path = remote.path();
        Command::new("git")
            .args(["init"])
            .current_dir(&remote_path)
            .output()
            .expect("git init");
        std::fs::write(remote_path.join("file.txt"), "first").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&remote_path)
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "first"])
            .current_dir(&remote_path)
            .output()
            .expect("git commit 1");
        std::fs::write(remote_path.join("file.txt"), "second").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&remote_path)
            .output()
            .expect("git add 2");
        Command::new("git")
            .args(["commit", "-m", "second"])
            .current_dir(&remote_path)
            .output()
            .expect("git commit 2");

        let clone = TempDir::new().expect("clone tempdir");
        clone_repo(&remote_path.to_string_lossy(), clone.path()).expect("clone_repo");

        let git = Git::new(clone.path(), OutputMode::Silent);
        let count = git.output(&["rev-list", "--count", "HEAD"]).unwrap();
        let cnt: usize = count.trim().parse().unwrap();
        assert!(cnt > 1, "expected full history, got {}", cnt);
    }

    #[test]
    fn fetch_updates_without_shallow() {
        // Remote repo with an initial commit
        let remote = TempDir::new().expect("remote tempdir");
        let remote_path = remote.path();
        Command::new("git")
            .args(["init"])
            .current_dir(&remote_path)
            .output()
            .expect("git init");
        std::fs::write(remote_path.join("a.txt"), "a").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&remote_path)
            .output()
            .expect("git add a");
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&remote_path)
            .output()
            .expect("git commit init");

        let clone = TempDir::new().expect("clone tempdir");
        clone_repo(&remote_path.to_string_lossy(), clone.path()).expect("clone_repo");

        std::fs::write(remote_path.join("b.txt"), "b").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&remote_path)
            .output()
            .expect("git add b");
        Command::new("git")
            .args(["commit", "-m", "new"])
            .current_dir(&remote_path)
            .output()
            .expect("git commit new");

        let branch_name = {
            let out = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(&remote_path)
                .output()
                .expect("rev-parse branch");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        let git = Git::new(clone.path(), OutputMode::Silent);
        git.fetch("origin", &branch_name).expect("fetch");
        git.merge_ff_only(&format!("origin/{}", branch_name))
            .expect("merge");

        let remote_head = {
            let out = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&remote_path)
                .output()
                .expect("rev-parse remote");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        let clone_head = git.rev_parse("HEAD").unwrap();
        assert_eq!(
            clone_head, remote_head,
            "clone HEAD should match remote after fetch"
        );
    }
}
