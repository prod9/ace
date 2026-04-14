use std::collections::HashMap;
use std::path::Path;

use crate::ace::Ace;
use crate::config;
use crate::glob;
use super::discover_skill::discover_skills;
use crate::state::skill_set::{ChangeKind, SkillSet};

pub struct UpdateSchool<'a> {
    pub school_root: &'a Path,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateSchoolError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] config::ConfigError),
    #[error("{0}")]
    Git(#[from] crate::git::GitError),
}

pub enum UpdateSchoolResult {
    NoImports,
    Updated {
        #[allow(dead_code)] // part of result API
        count: usize,
    },
}

impl UpdateSchool<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<UpdateSchoolResult, UpdateSchoolError> {
        let toml_path = self.school_root.join("school.toml");
        let school = config::school_toml::load(&toml_path)?;

        if school.imports.is_empty() {
            return Ok(UpdateSchoolResult::NoImports);
        }

        let by_source = group_by_source(&school.imports);
        let skills_dir = self.school_root.join("skills");
        let mut count = 0;

        for (source, patterns) in &by_source {
            let tmp = tempfile::tempdir()?;

            ace.progress(&format!("Fetching {source}"));
            if let Err(e) = crate::git::clone_github(source, tmp.path()) {
                ace.warn(&e.to_string());
                ace.hint(crate::git::auth_hint());
                return Err(e.into());
            }

            let discovered = discover_skills(tmp.path())?;
            let source_set = SkillSet::from_discovered(&discovered);

            for pattern in patterns {
                let names: Vec<&str> = if glob::is_glob(pattern) {
                    source_set.matching(pattern)
                } else {
                    vec![*pattern]
                };

                if names.is_empty() {
                    ace.warn(&format!("no skills matching {pattern} in {source}"));
                    continue;
                }

                let changes = source_set.copy_into(&skills_dir, &names)?;

                if changes.is_empty() && !glob::is_glob(pattern) {
                    ace.warn(&format!("skill {pattern} not found in {source}, skipping"));
                    continue;
                }

                for change in &changes {
                    let label = match change.kind {
                        ChangeKind::Added => "new",
                        ChangeKind::Modified => "updated",
                        ChangeKind::Removed => "removed",
                    };
                    ace.done(&format!("{} ({label})", change.name));
                }

                count += changes.len();
            }
        }

        ace.done(&format!("Updated {count} skill(s)"));
        Ok(UpdateSchoolResult::Updated { count })
    }
}

fn group_by_source(imports: &[config::school_toml::ImportDecl]) -> HashMap<&str, Vec<&str>> {
    let mut by_source: HashMap<&str, Vec<&str>> = HashMap::new();
    for imp in imports {
        by_source.entry(imp.source.as_str())
            .or_default()
            .push(imp.skill.as_str());
    }
    by_source
}
