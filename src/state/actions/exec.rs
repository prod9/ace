use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::config::backend::Backend;
use crate::session::Session;

pub struct Exec {
    pub backend: Backend,
    pub session_prompt: String,
    pub project_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub backend_args: Vec<String>,
}

impl Exec {
    pub fn run(&self, _session: &mut Session<'_>) -> Result<(), std::io::Error> {
        let mut cmd = Command::new(self.backend.binary());
        cmd.current_dir(&self.project_dir);

        for (key, val) in &self.env {
            cmd.env(key, val);
        }

        cmd.arg("--system-prompt").arg(&self.session_prompt);
        cmd.args(&self.backend_args);

        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        Err(err)
    }
}
