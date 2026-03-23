use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::ace::Ace;
use crate::config::backend::Backend;

pub struct Exec {
    pub backend: Backend,
    pub session_prompt: String,
    pub project_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub backend_args: Vec<String>,
}

impl Exec {
    pub fn run(&self, _ace: &mut Ace) -> Result<(), std::io::Error> {
        if self.backend == Backend::Flaude {
            return Ok(());
        }

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
