use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("git {cmd}: {source}")]
    Exec { cmd: String, source: std::io::Error },
    #[error("git {cmd}: {status}")]
    Exit { cmd: String, status: ExitStatus },
}

fn run(repo: &Path, args: &[&str]) -> Result<(), GitError> {
    let cmd_str = format!("git {}", args.join(" "));

    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| GitError::Exec { cmd: cmd_str.clone(), source: e })?;

    if !status.success() {
        return Err(GitError::Exit { cmd: cmd_str, status });
    }
    Ok(())
}

fn output(repo: &Path, args: &[&str]) -> Result<String, GitError> {
    let cmd_str = format!("git {}", args.join(" "));

    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| GitError::Exec { cmd: cmd_str.clone(), source: e })?;

    if !out.status.success() {
        return Err(GitError::Exit { cmd: cmd_str, status: out.status });
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn is_dirty(repo: &Path) -> Result<bool, GitError> {
    let out = output(repo, &["status", "--porcelain"])?;
    Ok(!out.is_empty())
}

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

pub fn fetch_shallow(repo: &Path, remote: &str, branch: &str) -> Result<(), GitError> {
    run(repo, &["fetch", "--depth", "1", "--no-tags", remote, branch])
}

pub fn reset_hard(repo: &Path, target: &str) -> Result<(), GitError> {
    run(repo, &["reset", "--hard", target])
}

pub fn diff_name_status(
    repo: &Path,
    from: &str,
    to: &str,
    path_filter: Option<&str>,
) -> Result<String, GitError> {
    let mut args = vec!["diff", "--name-status", from, to];
    if let Some(filter) = path_filter {
        args.push("--");
        args.push(filter);
    }
    output(repo, &args)
}

pub fn checkout_new_branch(repo: &Path, branch: &str) -> Result<(), GitError> {
    run(repo, &["checkout", "-b", branch])
}

pub fn checkout(repo: &Path, branch: &str) -> Result<(), GitError> {
    run(repo, &["checkout", branch])
}

pub fn add_all(repo: &Path) -> Result<(), GitError> {
    run(repo, &["add", "-A"])
}

pub fn commit(repo: &Path, message: &str) -> Result<(), GitError> {
    run(repo, &["commit", "-m", message])
}

pub fn diff(repo: &Path) -> Result<String, GitError> {
    output(repo, &["diff"])
}

pub fn push_new_branch(repo: &Path, remote: &str, branch: &str) -> Result<(), GitError> {
    run(repo, &["push", "-u", remote, branch])
}
