use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use crate::ace::OutputMode;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git {cmd}: {source}")]
    Exec { cmd: String, source: std::io::Error },
    #[error("git {cmd}: {status}{}", if stderr.is_empty() { String::new() } else { format!("\n{stderr}") })]
    Exit {
        cmd: String,
        status: ExitStatus,
        stderr: String,
    },
}

/// Build a `git` Command with non-interactive env so we fail fast instead of hanging
/// on credential or known_hosts prompts. Credential helpers (keychain, gh, etc.) still work.
fn git_command() -> Command {
    let mut cmd = Command::new("git");
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env(
        "GIT_SSH_COMMAND",
        "ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new",
    );
    cmd
}

/// Hint printed alongside git failures that look like auth/transport issues.
/// Points users at the two supported auth paths: SSH keys or the GitHub CLI.
pub fn auth_hint() -> &'static str {
    "If this is an auth issue, either:\n  \
     • Set up an SSH key and add it to GitHub:\n      \
     https://docs.github.com/en/authentication/connecting-to-github-with-ssh\n  \
     • Or install GitHub CLI and sign in:\n      \
     brew install gh && gh auth login"
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

        let out = git_command()
            .args(args)
            .current_dir(self.repo)
            .stdout(Stdio::null())
            .output()
            .map_err(|e| GitError::Exec {
                cmd: cmd_str.clone(),
                source: e,
            })?;

        if !out.status.success() {
            return Err(GitError::Exit {
                cmd: cmd_str,
                status: out.status,
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
            });
        }
        Ok(())
    }

    fn output(&self, args: &[&str]) -> Result<String, GitError> {
        let cmd_str = args.join(" ");

        let out = git_command()
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
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
            });
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// Normalize a GitHub source: strip URL prefix and `.git` suffix.
/// Accepts `https://github.com/owner/repo`, `https://github.com/owner/repo.git`,
/// or plain `owner/repo`. Returns `owner/repo`.
pub fn normalize_github_source(source: &str) -> String {
    let s = source
        .strip_prefix("https://github.com/")
        .or_else(|| source.strip_prefix("http://github.com/"))
        .unwrap_or(source);
    let s = s.strip_suffix(".git").unwrap_or(s);
    s.trim_end_matches('/').to_string()
}

/// Clone a GitHub repo by `owner/repo` specifier into `dest` using a full clone.
/// Accepts full GitHub URLs or plain `owner/repo` specifiers.
pub fn clone_github(source: &str, dest: &Path) -> Result<(), GitError> {
    let normalized = normalize_github_source(source);
    let url = format!("https://github.com/{normalized}.git");
    clone_repo(&url, dest)
}

pub fn ls_remote_tags(repo_url: &str, tag_filter: &str) -> Result<Vec<String>, GitError> {
    let cmd_str = format!("ls-remote --tags {repo_url} {tag_filter}");
    let out = git_command()
        .args(["ls-remote", "--tags", repo_url, tag_filter])
        .output()
        .map_err(|e| GitError::Exec {
            cmd: cmd_str.clone(),
            source: e,
        })?;

    if !out.status.success() {
        return Err(GitError::Exit {
            cmd: cmd_str,
            status: out.status,
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let tags = stdout
        .lines()
        .filter_map(|line| {
            let refname = line.split('\t').nth(1)?;
            let tag = refname.strip_prefix("refs/tags/")?;
            if tag.ends_with("^{}") {
                return None;
            }
            Some(tag.to_string())
        })
        .collect();
    Ok(tags)
}

/// Standalone — no repo context needed.
/// Performs a full clone (no `--depth`).
pub fn clone_repo(url: &str, dest: &Path) -> Result<(), GitError> {
    let cmd_str = format!("clone --no-tags {url}");

    let out = git_command()
        .args(["clone", "--no-tags", url])
        .arg(dest)
        .stdout(Stdio::null())
        .output()
        .map_err(|e| GitError::Exec {
            cmd: cmd_str.clone(),
            source: e,
        })?;

    if !out.status.success() {
        return Err(GitError::Exit {
            cmd: cmd_str,
            status: out.status,
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
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
    fn git_command_sets_noninteractive_env() {
        let cmd = git_command();
        let envs: Vec<(String, String)> = cmd
            .get_envs()
            .filter_map(|(k, v)| {
                v.map(|v| (k.to_string_lossy().into_owned(), v.to_string_lossy().into_owned()))
            })
            .collect();

        let prompt = envs.iter().find(|(k, _)| k == "GIT_TERMINAL_PROMPT");
        assert_eq!(prompt.map(|(_, v)| v.as_str()), Some("0"));

        let ssh = envs.iter().find(|(k, _)| k == "GIT_SSH_COMMAND");
        let ssh_val = ssh.map(|(_, v)| v.as_str()).unwrap_or("");
        assert!(ssh_val.contains("BatchMode=yes"), "GIT_SSH_COMMAND: {ssh_val}");
        assert!(
            ssh_val.contains("StrictHostKeyChecking=accept-new"),
            "GIT_SSH_COMMAND: {ssh_val}"
        );
    }

    #[test]
    fn normalize_plain_specifier() {
        assert_eq!(normalize_github_source("owner/repo"), "owner/repo");
    }

    #[test]
    fn normalize_strips_https_prefix() {
        assert_eq!(
            normalize_github_source("https://github.com/owner/repo"),
            "owner/repo"
        );
    }

    #[test]
    fn normalize_strips_git_suffix() {
        assert_eq!(
            normalize_github_source("owner/repo.git"),
            "owner/repo"
        );
    }

    #[test]
    fn normalize_strips_both() {
        assert_eq!(
            normalize_github_source("https://github.com/owner/repo.git"),
            "owner/repo"
        );
    }

    #[test]
    fn normalize_strips_trailing_slash() {
        assert_eq!(
            normalize_github_source("https://github.com/owner/repo/"),
            "owner/repo"
        );
    }

    #[test]
    fn normalize_http_prefix() {
        assert_eq!(
            normalize_github_source("http://github.com/owner/repo"),
            "owner/repo"
        );
    }

    #[test]
    fn normalize_preserves_dot_specifier() {
        assert_eq!(normalize_github_source("."), ".");
    }

    #[test]
    fn ls_remote_tags_local_repo() {
        let remote = TempDir::new().expect("remote tempdir");
        let remote_path = remote.path();
        Command::new("git")
            .args(["init"])
            .current_dir(remote_path)
            .output()
            .expect("git init");
        std::fs::write(remote_path.join("f.txt"), "x").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(remote_path)
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(remote_path)
            .output()
            .expect("git commit");
        Command::new("git")
            .args(["tag", "v0.1.0"])
            .current_dir(remote_path)
            .output()
            .expect("git tag v0.1.0");
        Command::new("git")
            .args(["tag", "v0.2.0"])
            .current_dir(remote_path)
            .output()
            .expect("git tag v0.2.0");
        Command::new("git")
            .args(["tag", "unrelated"])
            .current_dir(remote_path)
            .output()
            .expect("git tag unrelated");

        let url = remote_path.to_string_lossy();
        let tags = ls_remote_tags(&url, "v0.*").expect("ls_remote_tags");
        assert!(tags.contains(&"v0.1.0".to_string()));
        assert!(tags.contains(&"v0.2.0".to_string()));
        assert!(!tags.contains(&"unrelated".to_string()));
    }

    #[test]
    fn clone_repo_full_history() {
        // Remote repo with two commits
        let remote = TempDir::new().expect("remote tempdir");
        let remote_path = remote.path();
        Command::new("git")
            .args(["init"])
            .current_dir(remote_path)
            .output()
            .expect("git init");
        std::fs::write(remote_path.join("file.txt"), "first").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(remote_path)
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "first"])
            .current_dir(remote_path)
            .output()
            .expect("git commit 1");
        std::fs::write(remote_path.join("file.txt"), "second").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(remote_path)
            .output()
            .expect("git add 2");
        Command::new("git")
            .args(["commit", "-m", "second"])
            .current_dir(remote_path)
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
            .current_dir(remote_path)
            .output()
            .expect("git init");
        std::fs::write(remote_path.join("a.txt"), "a").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(remote_path)
            .output()
            .expect("git add a");
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(remote_path)
            .output()
            .expect("git commit init");

        let clone = TempDir::new().expect("clone tempdir");
        clone_repo(&remote_path.to_string_lossy(), clone.path()).expect("clone_repo");

        std::fs::write(remote_path.join("b.txt"), "b").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(remote_path)
            .output()
            .expect("git add b");
        Command::new("git")
            .args(["commit", "-m", "new"])
            .current_dir(remote_path)
            .output()
            .expect("git commit new");

        let branch_name = {
            let out = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(remote_path)
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
                .current_dir(remote_path)
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
