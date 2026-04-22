use std::path::Path;

use super::ace_toml::{self, AceToml};
use super::backend::Backend;
use super::paths::AcePaths;
use super::school_paths::{self, SchoolPaths};
use super::school_toml::{self, SchoolToml};
use super::ConfigError;

/// Raw config layers, preserved for write-back and inspection.
#[derive(Clone)]
pub struct Tree {
    pub ace_user: AceToml,
    pub ace_project: AceToml,
    pub ace_local: AceToml,
    /// Backend from school.toml, applied after load when school is known.
    pub school_backend: Option<Backend>,
    pub school_toml: Option<SchoolToml>,
    pub school_paths: Option<SchoolPaths>,
}

impl Tree {
    pub fn load(paths: &AcePaths) -> Result<Self, ConfigError> {
        let ace_user = load_or_default(&paths.user)?;
        let ace_project = load_or_default(&paths.project)?;
        let ace_local = load_or_default(&paths.local)?;

        // User layer alone doesn't mean a project is set up.
        let any_found = [&paths.project, &paths.local]
            .iter()
            .any(|p| p.exists());
        if !any_found {
            return Err(ConfigError::NoConfig);
        }

        Ok(Tree {
            ace_user,
            ace_project,
            ace_local,
            school_backend: None,
            school_toml: None,
            school_paths: None,
        })
    }

    /// Resolve school specifier from ace.toml layers (last non-empty wins).
    pub fn specifier(&self) -> Option<String> {
        [&self.ace_local, &self.ace_project, &self.ace_user]
            .iter()
            .find(|l| !l.school.is_empty())
            .map(|l| l.school.clone())
    }

    // Wired into production paths in step 2 (filter + resolution).
    #[allow(dead_code)]
    pub fn effective_skills(&self) -> Vec<String> {
        [&self.ace_local, &self.ace_project, &self.ace_user]
            .iter()
            .find(|l| !l.skills.is_empty())
            .map(|l| l.skills.clone())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn effective_include_skills(&self) -> Vec<String> {
        union_dedup([
            &self.ace_user.include_skills,
            &self.ace_project.include_skills,
            &self.ace_local.include_skills,
        ])
    }

    #[allow(dead_code)]
    pub fn effective_exclude_skills(&self) -> Vec<String> {
        union_dedup([
            &self.ace_user.exclude_skills,
            &self.ace_project.exclude_skills,
            &self.ace_local.exclude_skills,
        ])
    }

    /// Second pass: load school.toml + school_paths from the resolved specifier.
    pub fn load_school(&mut self, project_dir: &Path) -> Result<(), ConfigError> {
        let Some(spec) = self.specifier() else {
            return Ok(());
        };

        let sp = school_paths::resolve(project_dir, &spec)?;
        let school_toml_path = sp.root.join("school.toml");
        if school_toml_path.exists()
            && let Ok(st) = school_toml::load(&school_toml_path)
        {
            self.school_backend = st.backend;
            self.school_toml = Some(st);
        }
        self.school_paths = Some(sp);
        Ok(())
    }
}

fn union_dedup(lists: [&[String]; 3]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for list in lists {
        for s in list {
            if seen.insert(s.clone()) {
                out.push(s.clone());
            }
        }
    }
    out
}

fn load_or_default(path: &Path) -> Result<AceToml, ConfigError> {
    match ace_toml::load(path) {
        Ok(config) => Ok(config),
        Err(ConfigError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(AceToml::default())
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ace(skills: &[&str], include: &[&str], exclude: &[&str]) -> AceToml {
        AceToml {
            skills: skills.iter().map(|s| s.to_string()).collect(),
            include_skills: include.iter().map(|s| s.to_string()).collect(),
            exclude_skills: exclude.iter().map(|s| s.to_string()).collect(),
            ..AceToml::default()
        }
    }

    fn tree(user: AceToml, project: AceToml, local: AceToml) -> Tree {
        Tree {
            ace_user: user,
            ace_project: project,
            ace_local: local,
            school_backend: None,
            school_toml: None,
            school_paths: None,
        }
    }

    // skills: last-wins replace (local > project > user); empty at all = empty.

    #[test]
    fn effective_skills_all_empty() {
        let t = tree(AceToml::default(), AceToml::default(), AceToml::default());
        assert!(t.effective_skills().is_empty());
    }

    #[test]
    fn effective_skills_user_only() {
        let t = tree(ace(&["a"], &[], &[]), AceToml::default(), AceToml::default());
        assert_eq!(t.effective_skills(), vec!["a"]);
    }

    #[test]
    fn effective_skills_project_overrides_user() {
        let t = tree(ace(&["a"], &[], &[]), ace(&["b"], &[], &[]), AceToml::default());
        assert_eq!(t.effective_skills(), vec!["b"]);
    }

    #[test]
    fn effective_skills_local_overrides_project() {
        let t = tree(
            ace(&["a"], &[], &[]),
            ace(&["b"], &[], &[]),
            ace(&["c"], &[], &[]),
        );
        assert_eq!(t.effective_skills(), vec!["c"]);
    }

    // include_skills: union user → project → local, dedup, stable order.

    #[test]
    fn effective_include_skills_unions_all_scopes() {
        let t = tree(
            ace(&[], &["a"], &[]),
            ace(&[], &["b"], &[]),
            ace(&[], &["c"], &[]),
        );
        assert_eq!(t.effective_include_skills(), vec!["a", "b", "c"]);
    }

    #[test]
    fn effective_include_skills_dedups_preserving_first_occurrence() {
        let t = tree(
            ace(&[], &["a", "b"], &[]),
            ace(&[], &["b", "c"], &[]),
            ace(&[], &["a", "d"], &[]),
        );
        assert_eq!(t.effective_include_skills(), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn effective_include_skills_empty_when_all_empty() {
        let t = tree(AceToml::default(), AceToml::default(), AceToml::default());
        assert!(t.effective_include_skills().is_empty());
    }

    // exclude_skills: same merge as include_skills.

    #[test]
    fn effective_exclude_skills_unions_all_scopes() {
        let t = tree(
            ace(&[], &[], &["x"]),
            ace(&[], &[], &["y"]),
            ace(&[], &[], &["z"]),
        );
        assert_eq!(t.effective_exclude_skills(), vec!["x", "y", "z"]);
    }

    #[test]
    fn effective_exclude_skills_dedups_preserving_first_occurrence() {
        let t = tree(
            ace(&[], &[], &["x", "y"]),
            ace(&[], &[], &["y", "z"]),
            ace(&[], &[], &[]),
        );
        assert_eq!(t.effective_exclude_skills(), vec!["x", "y", "z"]);
    }
}
