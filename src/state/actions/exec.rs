use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::config::backend::Backend;
use crate::session::Session;

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("exec failed: {0}")]
    Exec(std::io::Error),
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

        cmd.arg("--system-prompt").arg(&self.session_prompt);

        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        Err(ExecError::Exec(err))
    }
}