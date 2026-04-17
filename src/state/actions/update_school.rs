use std::collections::HashMap;
use std::path::Path;

use crate::ace::Ace;
use crate::config;
use crate::glob;
use super::discover_skill::{discover_skills, Tier};
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

        for (source, decls) in &by_source {
            let tmp = tempfile::tempdir()?;

            ace.progress(&format!("Fetching {source}"));
            if let Err(e) = crate::git::clone_github(source, tmp.path()) {
                ace.warn(&e.to_string());
                ace.hint(crate::git::auth_hint());
                return Err(e.into());
            }

            let discovered = discover_skills(tmp.path())?;
            let source_set = SkillSet::from_discovered(&discovered);

            for decl in decls {
                let names = resolve_import_names(&source_set, decl);

                if names.is_empty() {
                    ace.warn(&format!("no skills matching {} in {source}", decl.skill));
                    continue;
                }

                let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
                let changes = source_set.copy_into(&skills_dir, &name_refs)?;

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

/// Resolve the list of skill names to copy for an import entry given a
/// discovered set from the source repo. Explicit names are looked up
/// across all tiers; glob patterns are tier-gated.
fn resolve_import_names(
    set: &SkillSet,
    decl: &config::school_toml::ImportDecl,
) -> Vec<String> {
    if glob::is_glob(&decl.skill) {
        let mut allowed = vec![Tier::Curated];
        if decl.include_experimental {
            allowed.push(Tier::Experimental);
        }
        if decl.include_system {
            allowed.push(Tier::System);
        }
        let filtered = set.filter_tiers(&allowed);
        filtered.matching(&decl.skill)
            .into_iter()
            .map(String::from)
            .collect()
    } else if set.names().any(|n| n == decl.skill) {
        vec![decl.skill.clone()]
    } else {
        Vec::new()
    }
}

fn group_by_source(
    imports: &[config::school_toml::ImportDecl],
) -> HashMap<&str, Vec<&config::school_toml::ImportDecl>> {
    let mut by_source: HashMap<&str, Vec<&config::school_toml::ImportDecl>> = HashMap::new();
    for imp in imports {
        by_source.entry(imp.source.as_str())
            .or_default()
            .push(imp);
    }
    by_source
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::school_toml::ImportDecl;
    use crate::state::actions::discover_skill::DiscoveredSkill;

    fn discovered(name: &str, tier: Tier) -> DiscoveredSkill {
        DiscoveredSkill {
            name: name.to_string(),
            path: std::path::PathBuf::from(format!("/tmp/{name}")),
            tier,
        }
    }

    fn import(skill: &str, experimental: bool, system: bool) -> ImportDecl {
        ImportDecl {
            skill: skill.to_string(),
            source: "owner/repo".to_string(),
            include_experimental: experimental,
            include_system: system,
        }
    }

    #[test]
    fn resolve_glob_matches_curated_by_default() {
        let set = SkillSet::from_discovered(&[
            discovered("alpha", Tier::Curated),
            discovered("beta",  Tier::Experimental),
            discovered("gamma", Tier::System),
        ]);
        let names = resolve_import_names(&set, &import("*", false, false));
        assert_eq!(names, vec!["alpha".to_string()]);
    }

    #[test]
    fn resolve_glob_with_experimental_flag_adds_that_tier() {
        let set = SkillSet::from_discovered(&[
            discovered("alpha", Tier::Curated),
            discovered("beta",  Tier::Experimental),
            discovered("gamma", Tier::System),
        ]);
        let mut names = resolve_import_names(&set, &import("*", true, false));
        names.sort();
        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
    }

    #[test]
    fn resolve_glob_with_both_flags_adds_all_tiers() {
        let set = SkillSet::from_discovered(&[
            discovered("alpha", Tier::Curated),
            discovered("beta",  Tier::Experimental),
            discovered("gamma", Tier::System),
        ]);
        let mut names = resolve_import_names(&set, &import("*", true, true));
        names.sort();
        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]);
    }

    #[test]
    fn resolve_explicit_name_finds_skill_in_any_tier() {
        let set = SkillSet::from_discovered(&[
            discovered("shell", Tier::Experimental),
        ]);
        let names = resolve_import_names(&set, &import("shell", false, false));
        assert_eq!(names, vec!["shell".to_string()]);
    }

    #[test]
    fn resolve_explicit_name_finds_skill_in_system_tier() {
        let set = SkillSet::from_discovered(&[
            discovered("skill-creator", Tier::System),
        ]);
        let names = resolve_import_names(&set, &import("skill-creator", false, false));
        assert_eq!(names, vec!["skill-creator".to_string()]);
    }

    #[test]
    fn resolve_explicit_name_missing_returns_empty() {
        let set = SkillSet::from_discovered(&[
            discovered("alpha", Tier::Curated),
        ]);
        let names = resolve_import_names(&set, &import("missing", false, false));
        assert!(names.is_empty());
    }

    #[test]
    fn resolve_glob_no_matches_returns_empty() {
        let set = SkillSet::from_discovered(&[
            discovered("alpha", Tier::Experimental),
        ]);
        let names = resolve_import_names(&set, &import("*", false, false));
        assert!(names.is_empty(), "curated-only default should not match experimental");
    }
}
