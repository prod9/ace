use std::path::{Path, PathBuf};

use crate::ace::Ace;
use crate::config;
use crate::config::school_toml::ImportDecl;

pub struct ImportSkill<'a> {
    pub source: &'a str,
    pub skill: Option<&'a str>,
    pub school_root: &'a Path,
}

pub struct DiscoveredSkill {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("clone failed: {0}")]
    Clone(String),
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
        clone_repo(self.source, tmp.path())?;

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

        copy_dir_recursive(&skill.path, &dest)?;

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

pub fn clone_repo(source: &str, dest: &Path) -> Result<(), ImportError> {
    let url = format!("https://github.com/{source}.git");
    crate::git::clone_shallow(&url, dest)
        .map_err(|e| ImportError::Clone(e.to_string()))
}

/// Discover skills by finding SKILL.md files in the repo.
/// Searches both root-level dirs and `skills/` subdirectory.
pub fn discover_skills(dir: &Path) -> Result<Vec<DiscoveredSkill>, ImportError> {
    let mut skills = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Check `skills/` subdirectory first (preferred convention)
    let skills_dir = dir.join("skills");
    if skills_dir.is_dir() {
        scan_for_skills(&skills_dir, &mut skills, &mut seen)?;
    }

    // Also check root-level directories
    scan_for_skills(dir, &mut skills, &mut seen)?;

    Ok(skills)
}

fn scan_for_skills(
    parent: &Path,
    skills: &mut Vec<DiscoveredSkill>,
    seen: &mut std::collections::HashSet<String>,
) -> Result<(), ImportError> {
    for entry in std::fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if name.starts_with('.') {
            continue;
        }

        if path.join("SKILL.md").exists() && seen.insert(name.clone()) {
            skills.push(DiscoveredSkill { name, path });
        }
    }
    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
