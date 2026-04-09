use std::collections::HashMap;
use std::path::Path;

use crate::ace::Ace;
use crate::config;
use super::discover_skill::{DiscoveredSkill, discover_skills};

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
        // -- load config --

        let toml_path = self.school_root.join("school.toml");
        let school = config::school_toml::load(&toml_path)?;

        if school.imports.is_empty() {
            return Ok(UpdateSchoolResult::NoImports);
        }

        // -- group imports by source --

        let by_source = group_by_source(&school.imports);

        // -- fetch each source and copy skills --

        let mut count = 0;

        for (source, skill_names) in &by_source {
            let tmp = tempfile::tempdir()?;

            ace.progress(&format!("Fetching {source}"));
            crate::git::clone_github(source, tmp.path())?;

            let discovered = discover_skills(tmp.path())?;
            count += copy_matching_skills(ace, self.school_root, source, &skill_names, &discovered)?;
        }

        ace.done(&format!("Updated {count} skill(s)"));
        Ok(UpdateSchoolResult::Updated { count })
    }
}

fn group_by_source<'a>(imports: &'a [config::school_toml::ImportDecl]) -> HashMap<&'a str, Vec<&'a str>> {
    let mut by_source: HashMap<&str, Vec<&str>> = HashMap::new();
    for imp in imports {
        by_source.entry(imp.source.as_str())
            .or_default()
            .push(imp.skill.as_str());
    }
    by_source
}

fn copy_matching_skills(
    ace: &mut Ace,
    school_root: &Path,
    source: &str,
    names: &[&str],
    discovered: &[DiscoveredSkill],
) -> Result<usize, UpdateSchoolError> {
    let mut count = 0;

    for name in names {
        let Some(skill) = discovered.iter().find(|s| s.name == *name) else {
            ace.warn(&format!("skill {name} not found in {source}, skipping"));
            continue;
        };

        let dest = school_root.join("skills").join(name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }

        crate::fsutil::copy_dir_recursive(&skill.path, &dest)?;
        count += 1;
    }

    Ok(count)
}
