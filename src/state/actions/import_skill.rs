use std::path::Path;

use crate::ace::Ace;
use crate::config;
use crate::config::school_toml::ImportDecl;

use super::discover_skill::{DiscoveredSkill, discover_skills};

pub struct ImportSkill<'a> {
    pub source: &'a str,
    pub skill: Option<&'a str>,
    pub school_root: &'a Path,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("{0}")]
    Clone(#[from] crate::git::GitError),
    #[error("no skills found in {0}")]
    NoSkills(String),
    #[error("skill not found: {0}")]
    SkillNotFound(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] config::ConfigError),
}

/// Result of a successful import — or a request for the caller to pick a skill.
pub enum ImportResult {
    Done {
        #[allow(dead_code)] // part of result API
        skill: String,
    },
    NeedsSelection(Vec<DiscoveredSkill>),
}

impl ImportSkill<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<ImportResult, ImportError> {
        let tmp = tempfile::tempdir()?;

        ace.progress(&format!("Cloning {}", self.source));
        crate::git::clone_github(self.source, tmp.path())?;

        let skills = discover_skills(tmp.path())?;
        if skills.is_empty() {
            return Err(ImportError::NoSkills(self.source.to_string()));
        }

        let selected = match self.skill {
            Some(name) => {
                let found = skills.iter().find(|s| s.name == name)
                    .ok_or_else(|| ImportError::SkillNotFound(name.to_string()))?;
                found
            }
            None if skills.len() == 1 => &skills[0],
            None => return Ok(ImportResult::NeedsSelection(skills)),
        };

        self.install_skill(selected)?;

        ace.done(&format!("Imported skill: {}", selected.name));
        Ok(ImportResult::Done { skill: selected.name.clone() })
    }

    /// Install a specific discovered skill after selection.
    pub fn install_selected(&self, skill: &DiscoveredSkill, ace: &mut Ace) -> Result<(), ImportError> {
        ace.progress(&format!("Installing {}", skill.name));
        self.install_skill(skill)?;
        ace.done(&format!("Imported skill: {}", skill.name));
        Ok(())
    }

    fn install_skill(&self, skill: &DiscoveredSkill) -> Result<(), ImportError> {
        let dest = self.school_root.join("skills").join(&skill.name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }

        crate::fsutil::copy_dir_recursive(&skill.path, &dest)?;

        let toml_path = self.school_root.join("school.toml");
        let mut school = config::school_toml::load(&toml_path)?;

        let entry = school.imports.iter_mut().find(|i| i.skill == skill.name);
        match entry {
            Some(existing) => existing.source = self.source.to_string(),
            None => school.imports.push(ImportDecl {
                skill: skill.name.clone(),
                source: self.source.to_string(),
            }),
        }

        config::school_toml::save(&toml_path, &school)?;
        Ok(())
    }
}
