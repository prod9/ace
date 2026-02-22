use std::path::Path;

use crate::session::Session;

#[derive(Debug, thiserror::Error)]
pub enum SchoolInitError {
    #[error("not in git repo, git init?")]
    NotInGitRepo,
    #[error("school.toml already exists")]
    AlreadyExists,
    #[error("write failed: {0}")]
    Write(#[from] std::io::Error),
}

pub struct SchoolInit<'a> {
    pub name: &'a str,
    pub project_dir: &'a Path,
}

impl SchoolInit<'_> {
    pub fn run(&self, _session: &mut Session<'_>) -> Result<(), SchoolInitError> {
        if !super::is_git_repo(self.project_dir) {
            return Err(SchoolInitError::NotInGitRepo);
        }

        let toml_path = self.project_dir.join("school.toml");
        if toml_path.exists() {
            return Err(SchoolInitError::AlreadyExists);
        }

        let content = format!("[school]\nname = \"{}\"\n", self.name);
        std::fs::write(&toml_path, content)?;

        Ok(())
    }
}
