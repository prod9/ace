use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::session::Session;

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("no backend found: install `claude` or `opencode`")]
    NoBackend,
    #[error("exec failed: {0}")]
    Exec(std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    ClaudeCode,
    OpenCode,
}

impl Backend {
    pub fn binary(&self) -> &'static str {
        match self {
            Backend::ClaudeCode => "claude",
            Backend::OpenCode => "opencode",
        }
    }
}

pub struct Exec {
    pub backend: Backend,
    pub session_prompt: String,
    pub project_dir: PathBuf,
    pub env: HashMap<String, String>,
}

impl Exec {
    pub fn run(&self, _session: &mut Session<'_>) -> Result<(), ExecError> {
        let mut cmd = Command::new(self.backend.binary());
        cmd.current_dir(&self.project_dir);

        for (key, val) in &self.env {
            cmd.env(key, val);
        }

        match self.backend {
            Backend::ClaudeCode => {
                cmd.arg("--system-prompt").arg(&self.session_prompt);
            }
            Backend::OpenCode => {
                cmd.env("ACE_SYSTEM_PROMPT", &self.session_prompt);
            }
        }

        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        Err(ExecError::Exec(err))
    }
}

pub fn detect_backend() -> Result<Backend, ExecError> {
    if which("claude") {
        return Ok(Backend::ClaudeCode);
    }
    if which("opencode") {
        return Ok(Backend::OpenCode);
    }
    Err(ExecError::NoBackend)
}

fn which(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_binary_names() {
        assert_eq!(Backend::ClaudeCode.binary(), "claude");
        assert_eq!(Backend::OpenCode.binary(), "opencode");
    }

    #[test]
    fn detect_backend_finds_something_or_errors() {
        // In CI or environments without claude/opencode, this should return NoBackend.
        // We just verify it doesn't panic.
        let _result = detect_backend();
    }
}
