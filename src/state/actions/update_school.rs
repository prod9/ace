use std::collections::HashMap;
use std::path::Path;

use crate::ace::Ace;
use crate::config;
use crate::glob;
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
        let toml_path = self.school_root.join("school.toml");
        let school = config::school_toml::load(&toml_path)?;

        if school.imports.is_empty() {
            return Ok(UpdateSchoolResult::NoImports);
        }

        let by_source = group_by_source(&school.imports);
        let mut count = 0;

        for (source, patterns) in &by_source {
            let tmp = tempfile::tempdir()?;

            ace.progress(&format!("Fetching {source}"));
            crate::git::clone_github(source, tmp.path())?;

            let discovered = discover_skills(tmp.path())?;
            count += copy_matching_skills(ace, self.school_root, source, patterns, &discovered)?;
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
    patterns: &[&str],
    discovered: &[DiscoveredSkill],
) -> Result<usize, UpdateSchoolError> {
    let mut count = 0;

    for pattern in patterns {
        if glob::is_glob(pattern) {
            count += copy_glob_skills(ace, school_root, source, pattern, discovered)?;
        } else {
            count += copy_exact_skill(ace, school_root, source, pattern, discovered)?;
        }
    }

    Ok(count)
}

fn copy_exact_skill(
    ace: &mut Ace,
    school_root: &Path,
    source: &str,
    name: &str,
    discovered: &[DiscoveredSkill],
) -> Result<usize, UpdateSchoolError> {
    let Some(skill) = discovered.iter().find(|s| s.name == name) else {
        ace.warn(&format!("skill {name} not found in {source}, skipping"));
        return Ok(0);
    };

    let dest = school_root.join("skills").join(name);
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }

    crate::fsutil::copy_dir_recursive(&skill.path, &dest)?;
    Ok(1)
}

fn copy_glob_skills(
    ace: &mut Ace,
    school_root: &Path,
    source: &str,
    pattern: &str,
    discovered: &[DiscoveredSkill],
) -> Result<usize, UpdateSchoolError> {
    let matched: Vec<&DiscoveredSkill> = discovered.iter()
        .filter(|s| glob::glob_match(pattern, &s.name))
        .collect();

    if matched.is_empty() {
        ace.warn(&format!("no skills matching {pattern} in {source}"));
        return Ok(0);
    }

    let mut count = 0;
    for skill in matched {
        let dest = school_root.join("skills").join(&skill.name);
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }

        crate::fsutil::copy_dir_recursive(&skill.path, &dest)?;
        count += 1;
    }

    Ok(count)
}
