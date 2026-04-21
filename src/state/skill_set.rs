use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::actions::{DiscoveredSkill, Tier};

#[derive(Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug)]
pub struct SkillChange {
    pub name: String,
    pub kind: ChangeKind,
}

#[derive(Clone)]
struct Entry {
    path: PathBuf,
    tier: Tier,
}

/// A set of skills discovered from a directory or import source.
pub struct SkillSet {
    skills: HashMap<String, Entry>,
}

impl SkillSet {
    /// Scan a directory for subdirs containing SKILL.md. All skills are
    /// tagged Tier::Curated — this constructor is for flat layouts where
    /// tier distinctions don't apply.
    #[allow(dead_code)] // used in tests; production use coming via update_cache migration
    pub fn from_dir(dir: &Path) -> Result<Self, std::io::Error> {
        let mut skills = HashMap::new();

        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(Self { skills });
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }
            if !path.join("SKILL.md").exists() {
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            skills.insert(name.to_string(), Entry { path, tier: Tier::Curated });
        }

        Ok(Self { skills })
    }

    /// Build from discover_skills output.
    pub fn from_discovered(discovered: &[DiscoveredSkill]) -> Self {
        let skills = discovered.iter()
            .map(|s| (s.name.clone(), Entry { path: s.path.clone(), tier: s.tier }))
            .collect();
        Self { skills }
    }

    /// Return a new set containing only skills whose tier is in `allowed`.
    pub fn filter_tiers(&self, allowed: &[Tier]) -> Self {
        let skills = self.skills.iter()
            .filter(|(_, e)| allowed.contains(&e.tier))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Self { skills }
    }

    /// All skill names in the set.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.skills.keys().map(String::as_str)
    }

    /// Skill names matching a glob pattern.
    pub fn matching(&self, pattern: &str) -> Vec<&str> {
        self.names()
            .filter(|name| crate::glob::glob_match(pattern, name))
            .collect()
    }

    /// Copy named skills into dest_dir, returning what changed.
    ///
    /// Each skill is classified as Added (didn't exist at dest) or Modified
    /// (overwrote existing). Skills not in this set are silently skipped.
    pub fn copy_into(
        &self,
        dest_dir: &Path,
        names: &[&str],
    ) -> Result<Vec<SkillChange>, std::io::Error> {
        let mut changes = Vec::new();

        for &name in names {
            let Some(entry) = self.skills.get(name) else {
                continue;
            };

            let dest = dest_dir.join(name);
            let kind = if dest.exists() {
                std::fs::remove_dir_all(&dest)?;
                ChangeKind::Modified
            } else {
                ChangeKind::Added
            };

            crate::fsutil::copy_dir_recursive(&entry.path, &dest)?;
            changes.push(SkillChange { name: name.to_string(), kind });
        }

        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_skill(dir: &Path, name: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).expect("mkdir");
        fs::write(skill_dir.join("SKILL.md"), format!("# {name}")).expect("write");
    }

    #[test]
    fn from_dir_discovers_skills() {
        let tmp = tempfile::tempdir().expect("tempdir");
        make_skill(tmp.path(), "go-coding");
        make_skill(tmp.path(), "rust-coding");
        fs::create_dir(tmp.path().join("not-a-skill")).expect("mkdir");

        let set = SkillSet::from_dir(tmp.path()).expect("from_dir");
        let mut names: Vec<&str> = set.names().collect();
        names.sort();
        assert_eq!(names, vec!["go-coding", "rust-coding"]);
    }

    #[test]
    fn from_dir_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let set = SkillSet::from_dir(tmp.path()).expect("from_dir");
        assert_eq!(set.names().count(), 0);
    }

    #[test]
    fn from_dir_missing() {
        let set = SkillSet::from_dir(Path::new("/nonexistent")).expect("from_dir");
        assert_eq!(set.names().count(), 0);
    }

    #[test]
    fn matching_glob() {
        let tmp = tempfile::tempdir().expect("tempdir");
        make_skill(tmp.path(), "go-coding");
        make_skill(tmp.path(), "rust-coding");
        make_skill(tmp.path(), "frontend-design");

        let set = SkillSet::from_dir(tmp.path()).expect("from_dir");

        let mut coding: Vec<&str> = set.matching("*-coding");
        coding.sort();
        assert_eq!(coding, vec!["go-coding", "rust-coding"]);

        let all: Vec<&str> = set.matching("*");
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn copy_into_adds_new_skill() {
        let src = tempfile::tempdir().expect("src");
        let dest = tempfile::tempdir().expect("dest");
        make_skill(src.path(), "my-skill");

        let set = SkillSet::from_dir(src.path()).expect("from_dir");
        let changes = set.copy_into(dest.path(), &["my-skill"]).expect("copy");

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].name, "my-skill");
        assert_eq!(changes[0].kind, ChangeKind::Added);
        assert!(dest.path().join("my-skill/SKILL.md").exists());
    }

    #[test]
    fn copy_into_modifies_existing() {
        let src = tempfile::tempdir().expect("src");
        let dest = tempfile::tempdir().expect("dest");
        make_skill(src.path(), "my-skill");
        make_skill(dest.path(), "my-skill");

        let set = SkillSet::from_dir(src.path()).expect("from_dir");
        let changes = set.copy_into(dest.path(), &["my-skill"]).expect("copy");

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind, ChangeKind::Modified);
    }

    #[test]
    fn copy_into_skips_unknown() {
        let src = tempfile::tempdir().expect("src");
        let dest = tempfile::tempdir().expect("dest");
        make_skill(src.path(), "my-skill");

        let set = SkillSet::from_dir(src.path()).expect("from_dir");
        let changes = set.copy_into(dest.path(), &["nonexistent"]).expect("copy");
        assert!(changes.is_empty());
    }

    // -- tier filtering (PROD9-75) --

    fn discovered(name: &str, tier: Tier) -> DiscoveredSkill {
        DiscoveredSkill {
            name: name.to_string(),
            path: PathBuf::from(name),
            tier,
        }
    }

    #[test]
    fn filter_tiers_keeps_only_allowed() {
        let set = SkillSet::from_discovered(&[
            discovered("cur",  Tier::Curated),
            discovered("exp",  Tier::Experimental),
            discovered("sys",  Tier::System),
        ]);

        let filtered = set.filter_tiers(&[Tier::Curated]);
        let names: Vec<&str> = filtered.names().collect();
        assert_eq!(names, vec!["cur"]);
    }

    #[test]
    fn filter_tiers_multiple_allowed() {
        let set = SkillSet::from_discovered(&[
            discovered("cur",  Tier::Curated),
            discovered("exp",  Tier::Experimental),
            discovered("sys",  Tier::System),
        ]);

        let filtered = set.filter_tiers(&[Tier::Curated, Tier::Experimental]);
        let mut names: Vec<&str> = filtered.names().collect();
        names.sort();
        assert_eq!(names, vec!["cur", "exp"]);
    }

    #[test]
    fn filter_tiers_empty_allowed_returns_empty() {
        let set = SkillSet::from_discovered(&[
            discovered("cur",  Tier::Curated),
        ]);

        let filtered = set.filter_tiers(&[]);
        assert_eq!(filtered.names().count(), 0);
    }

    #[test]
    fn matching_after_filter_respects_filter() {
        let set = SkillSet::from_discovered(&[
            discovered("a-cur", Tier::Curated),
            discovered("a-exp", Tier::Experimental),
            discovered("b-sys", Tier::System),
        ]);

        let filtered = set.filter_tiers(&[Tier::Curated]);
        let matched = filtered.matching("*");
        assert_eq!(matched, vec!["a-cur"]);
    }
}
