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
            flaude_record_exec(&self.backend_args)?;
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

/// Record the exec call to `FLAUDE_RECORD` for test assertions.
fn flaude_record_exec(backend_args: &[String]) -> Result<(), std::io::Error> {
    let record_path = match std::env::var("FLAUDE_RECORD") {
        Ok(p) => p,
        Err(_) => return Ok(()),
    };

    use std::io::Write;
    let record = serde_json::json!({
        "action": "exec",
        "backend_args": backend_args,
    });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&record_path)?;

    writeln!(file, "{record}")?;
    Ok(())
}
