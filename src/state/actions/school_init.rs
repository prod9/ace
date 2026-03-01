use std::path::Path;

use crate::ace::Ace;
use crate::prompts;

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
    pub force: bool,
}

impl SchoolInit<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<(), SchoolInitError> {
        if !super::is_git_repo(self.project_dir) {
            return Err(SchoolInitError::NotInGitRepo);
        }

        let toml_path = self.project_dir.join("school.toml");
        if !self.force && toml_path.exists() {
            return Err(SchoolInitError::AlreadyExists);
        }

        let content = format!("name = \"{}\"\n", self.name);
        std::fs::write(&toml_path, content)?;

        let instructions = self.project_dir.join("CLAUDE.md");
        if !instructions.exists() {
            let ctx = prompts::PromptCtx::new(Path::new(".claude"), self.name);
            let content = prompts::render(prompts::SCHOOL_CLAUDE_MD, &ctx);
            std::fs::write(&instructions, content)?;
            ace.done("Created CLAUDE.md");
        }

        let skill_dir = self.project_dir.join("skills").join("ace-school");
        let skill_path = skill_dir.join("SKILL.md");
        if !skill_path.exists() {
            std::fs::create_dir_all(&skill_dir)?;
            std::fs::write(&skill_path, prompts::ACE_SCHOOL_SKILL)?;
            ace.done("Created skills/ace-school/SKILL.md");
        }

        Ok(())
    }
}
