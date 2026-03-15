use std::path::Path;

use crate::ace::Ace;
use crate::config::school_toml;
use crate::config::ConfigError;
use crate::templates;

#[derive(Debug, thiserror::Error)]
pub enum SchoolInitError {
    #[error("not in git repo, git init?")]
    NotInGitRepo,
    #[error("school.toml already exists")]
    AlreadyExists,
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("write failed: {0}")]
    Write(std::io::Error),
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

        if self.force && toml_path.exists() {
            let mut toml = school_toml::load(&toml_path)?;
            toml.name = self.name.to_string();
            school_toml::save(&toml_path, &toml)?;
        } else {
            let toml = school_toml::SchoolToml {
                name: self.name.to_string(),
                ..Default::default()
            };
            school_toml::save(&toml_path, &toml)?;
        }
        ace.done("Created school.toml");

        let instructions = self.project_dir.join("CLAUDE.md");
        if !instructions.exists() {
            let ctx = templates::PromptCtx::new(Path::new(".claude"), self.name);
            let content = templates::render(templates::SCHOOL_CLAUDE_MD, &ctx);
            std::fs::write(&instructions, content).map_err(SchoolInitError::Write)?;
            ace.done("Created CLAUDE.md");
        }

        let readme = self.project_dir.join("README.md");
        if !readme.exists() {
            let ctx = templates::PromptCtx::new(Path::new(".claude"), self.name);
            let content = templates::render(templates::SCHOOL_README, &ctx);
            std::fs::write(&readme, content).map_err(SchoolInitError::Write)?;
            ace.done("Created README.md");
        }

        let skill_dir = self.project_dir.join("skills").join("ace-school");
        let skill_path = skill_dir.join("SKILL.md");
        if !skill_path.exists() {
            std::fs::create_dir_all(&skill_dir).map_err(SchoolInitError::Write)?;
            std::fs::write(&skill_path, templates::ACE_SCHOOL_SKILL)
                .map_err(SchoolInitError::Write)?;
            ace.done("Created skills/ace-school/SKILL.md");
        }

        Ok(())
    }
}
