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
    pub name: Option<&'a str>,
    pub project_dir: &'a Path,
}

impl SchoolInit<'_> {
    pub async fn run(&self, session: &mut Session<'_>) -> Result<(), SchoolInitError> {
        if !super::is_git_repo(self.project_dir) {
            return Err(SchoolInitError::NotInGitRepo);
        }

        let toml_path = self.project_dir.join("school.toml");
        if toml_path.exists() {
            return Err(SchoolInitError::AlreadyExists);
        }

        let name = match self.name {
            Some(n) => n.to_string(),
            None => session.ui.ask("School name:").await,
        };

        let content = format!("[school]\nname = \"{name}\"\n");
        std::fs::write(&toml_path, content)?;

        session.ui.message(&format!("Created {}", toml_path.display())).await;

        Ok(())
    }
}
