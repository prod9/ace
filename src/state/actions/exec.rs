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

        let args = build_exec_args(self.backend, &self.session_prompt, &self.backend_args);
        cmd.args(&args);

        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        Err(err)
    }
}

/// Build the argument list for the backend CLI invocation.
fn build_exec_args(backend: Backend, session_prompt: &str, backend_args: &[String]) -> Vec<String> {
    match backend {
        Backend::Codex => {
            let mut args = backend_args.to_vec();
            args.push("-c".to_string());
            args.push(format!(
                "developer_instructions={}",
                toml::Value::String(session_prompt.to_string())
            ));
            args
        }
        _ => {
            let mut args = vec![
                "--system-prompt".to_string(),
                session_prompt.to_string(),
            ];
            args.extend_from_slice(backend_args);
            args
        }
    }
}

/// Record the exec call to `$HOME/.flaude-exec-records.jsonl` for test assertions.
fn flaude_record_exec(backend_args: &[String]) -> Result<(), std::io::Error> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Ok(()),
    };

    use std::io::Write;
    let record_path = std::path::Path::new(&home).join(".flaude-exec-records.jsonl");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::backend::Backend;

    #[test]
    fn build_exec_args_includes_system_prompt() {
        let args = build_exec_args(Backend::Claude, "You are helpful.", &[]);
        assert_eq!(args, vec!["--system-prompt", "You are helpful."]);
    }

    #[test]
    fn build_exec_args_appends_backend_args() {
        let backend_args = vec!["--yolo".to_string(), "--verbose".to_string()];
        let args = build_exec_args(Backend::Claude, "prompt", &backend_args);
        assert_eq!(
            args,
            vec!["--system-prompt", "prompt", "--yolo", "--verbose"]
        );
    }

    #[test]
    fn build_exec_args_codex_does_not_send_prompt_as_user_message() {
        let backend_args = vec!["--full-auto".to_string()];
        let args = build_exec_args(Backend::Codex, "prompt", &backend_args);
        assert_eq!(
            args,
            vec![
                "--full-auto",
                "-c",
                "developer_instructions=\"prompt\"",
            ]
        );
    }
}
