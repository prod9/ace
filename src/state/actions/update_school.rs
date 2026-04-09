use std::collections::{HashMap, HashSet};
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

        let local_skills = local_skill_names(self.school_root, &school.imports)?;
        let by_source = group_by_source(&school.imports);

        let mut count = 0;

        for (source, patterns) in &by_source {
            let tmp = tempfile::tempdir()?;

            ace.progress(&format!("Fetching {source}"));
            crate::git::clone_github(source, tmp.path())?;

            let discovered = discover_skills(tmp.path())?;
            count += copy_matching_skills(ace, self.school_root, source, patterns, &discovered, &local_skills)?;
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

/// Collect skill names that exist locally and are NOT tracked in [[imports]].
/// These are the child school's own skills — never overwritten by wildcard imports.
fn local_skill_names(
    school_root: &Path,
    imports: &[config::school_toml::ImportDecl],
) -> Result<HashSet<String>, std::io::Error> {
    let imported: HashSet<&str> = imports.iter()
        .filter(|i| !glob::is_glob(&i.skill))
        .map(|i| i.skill.as_str())
        .collect();

    let skills_dir = school_root.join("skills");
    let mut local = HashSet::new();

    let Ok(entries) = std::fs::read_dir(&skills_dir) else {
        return Ok(local);
    };

    for entry in entries {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };

        if !imported.contains(name.as_str()) {
            local.insert(name);
        }
    }

    Ok(local)
}

fn copy_matching_skills(
    ace: &mut Ace,
    school_root: &Path,
    source: &str,
    patterns: &[&str],
    discovered: &[DiscoveredSkill],
    local_skills: &HashSet<String>,
) -> Result<usize, UpdateSchoolError> {
    let mut count = 0;

    for pattern in patterns {
        if glob::is_glob(pattern) {
            count += copy_glob_skills(ace, school_root, source, pattern, discovered, local_skills)?;
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
    local_skills: &HashSet<String>,
) -> Result<usize, UpdateSchoolError> {
    let matched: Vec<&DiscoveredSkill> = discovered.iter()
        .filter(|s| glob::glob_match(pattern, &s.name))
        .filter(|s| !local_skills.contains(&s.name))
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
