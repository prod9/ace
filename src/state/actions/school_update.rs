use std::collections::HashMap;
use std::path::Path;

use crate::config;
use crate::session::Session;
use super::import_skill::{clone_repo, copy_dir_recursive, discover_skills, ImportError};

pub struct SchoolUpdate<'a> {
    pub school_root: &'a Path,
}

#[derive(Debug, thiserror::Error)]
pub enum SchoolUpdateError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] config::ConfigError),
    #[error("{0}")]
    Import(#[from] ImportError),
}

pub enum SchoolUpdateResult {
    NoImports,
    Updated { count: usize },
}

impl SchoolUpdate<'_> {
    pub fn run(&self, session: &mut Session<'_>) -> Result<SchoolUpdateResult, SchoolUpdateError> {
        let toml_path = self.school_root.join("school.toml");
        let school = config::school_toml::load(&toml_path)?;

        if school.imports.is_empty() {
            return Ok(SchoolUpdateResult::NoImports);
        }

        // Group imports by source to avoid cloning the same repo twice.
        let mut by_source: HashMap<&str, Vec<&str>> = HashMap::new();
        for imp in &school.imports {
            by_source.entry(imp.source.as_str())
                .or_default()
                .push(imp.skill.as_str());
        }

        let mut count = 0;
        for (source, skill_names) in &by_source {
            let tmp = tempfile::tempdir()?;

            session.progress(&format!("Fetching {source}"));
            clone_repo(source, tmp.path())?;

            let discovered = discover_skills(tmp.path())?;
            for name in skill_names {
                let found = discovered.iter().find(|s| s.name == *name);
                let skill = match found {
                    Some(s) => s,
                    None => {
                        session.warn(&format!("skill {name} not found in {source}, skipping"));
                        continue;
                    }
                };

                let dest = self.school_root.join("skills").join(name);
                if dest.exists() {
                    std::fs::remove_dir_all(&dest)?;
                }
                copy_dir_recursive(&skill.path, &dest)?;
                count += 1;
            }
        }

        session.done(&format!("Updated {count} skill(s)"));
        Ok(SchoolUpdateResult::Updated { count })
    }
}

