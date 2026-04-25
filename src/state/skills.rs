//! Unified skills domain: typestate over discovery → resolution.
//!
//! `Skill<S>` carries name/path/tier from discovery onward; the marker `S`
//! adds resolver verdict + provenance trace once `resolve()` runs.
//! `Skills<S>` is the collection plus, when resolved, the resolution-wide
//! diagnostics (unknown patterns + collisions).
//!
//! Wraps the existing `state::discover` + `state::resolver` pure logic
//! during the migration. Old modules go away once all callers move here.

// `from_discovered` / `filter_tiers` / `matching` / `copy_into` / `names`
// land in step 4 (pull_imports + add_import migration). `included` /
// `diagnostics` land in step 3 (link_skills migration). Module-level allow
// keeps staged-integration warnings off the build until then.
#![allow(dead_code)]

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::config::tree::Tree;
use crate::state::discover::{self, DiscoveredSkill, Tier};
use crate::state::resolver::{self, Collision, Decision, Entry, UnknownPattern};
use crate::state::skill_set::{ChangeKind, SkillChange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Discovered;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub decision: Decision,
    pub trace: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct Skill<S> {
    pub name: String,
    pub path: PathBuf,
    pub tier: Tier,
    pub state: S,
}

#[derive(Debug, Clone, Default)]
pub struct Skills<S> {
    items: Vec<Skill<S>>,
    diagnostics: Diagnostics,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub unknown_patterns: Vec<UnknownPattern>,
    pub collisions: Vec<Collision>,
}

// ---- Skills<Discovered> ----

impl Skills<Discovered> {
    /// Walk the school's `skills/` tree. See `discover::discover_skills` for
    /// the tier priority order.
    pub fn discover(school_root: &Path) -> io::Result<Self> {
        Ok(Self::from_discovered(&discover::discover_skills(school_root)?))
    }

    pub fn from_discovered(discovered: &[DiscoveredSkill]) -> Self {
        let items = discovered
            .iter()
            .map(|d| Skill {
                name: d.name.clone(),
                path: d.path.clone(),
                tier: d.tier,
                state: Discovered,
            })
            .collect();
        Self { items, diagnostics: Diagnostics::default() }
    }

    /// Run the three-layer resolver against the given config tree.
    /// Consumes `self` — the typestate transition is one-way.
    pub fn resolve(self, tree: &Tree) -> Skills<Resolved> {
        let names: Vec<String> = self.items.iter().map(|s| s.name.clone()).collect();
        let resolution = resolver::resolve(
            &names,
            &tree.ace_user,
            &tree.ace_project,
            &tree.ace_local,
        );

        let mut by_name: HashMap<String, (PathBuf, Tier)> = self
            .items
            .into_iter()
            .map(|s| (s.name, (s.path, s.tier)))
            .collect();

        let items = resolution
            .skills
            .into_iter()
            .filter_map(|r| {
                let (path, tier) = by_name.remove(&r.name)?;
                Some(Skill {
                    name: r.name,
                    path,
                    tier,
                    state: Resolved {
                        decision: r.decision,
                        trace: r.trace,
                    },
                })
            })
            .collect();

        Skills {
            items,
            diagnostics: Diagnostics {
                unknown_patterns: resolution.unknown_patterns,
                collisions: resolution.collisions,
            },
        }
    }

    /// Keep only skills whose tier is in `allowed`.
    pub fn filter_tiers(&self, allowed: &[Tier]) -> Self {
        let items = self
            .items
            .iter()
            .filter(|s| allowed.contains(&s.tier))
            .cloned()
            .collect();
        Self { items, diagnostics: Diagnostics::default() }
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.items.iter().map(|s| s.name.as_str())
    }

    /// Skill names matching a glob pattern.
    pub fn matching(&self, pattern: &str) -> Vec<&str> {
        self.names()
            .filter(|name| crate::glob::glob_match(pattern, name))
            .collect()
    }

    /// Copy named skills into `dest_dir`. Each skill is classified Added
    /// (didn't exist) or Modified (overwrote). Unknown names silently skipped.
    pub fn copy_into(&self, dest_dir: &Path, names: &[&str]) -> io::Result<Vec<SkillChange>> {
        let by_name: HashMap<&str, &Skill<Discovered>> =
            self.items.iter().map(|s| (s.name.as_str(), s)).collect();

        let mut changes = Vec::new();
        for &name in names {
            let Some(skill) = by_name.get(name) else {
                continue;
            };

            let dest = dest_dir.join(name);
            let kind = if dest.exists() {
                std::fs::remove_dir_all(&dest)?;
                ChangeKind::Modified
            } else {
                ChangeKind::Added
            };

            crate::fsutil::copy_dir_recursive(&skill.path, &dest)?;
            changes.push(SkillChange { name: name.to_string(), kind });
        }
        Ok(changes)
    }
}

// ---- Skills<Resolved> ----

impl Skills<Resolved> {
    pub fn find(&self, name: &str) -> Option<&Skill<Resolved>> {
        self.items.iter().find(|s| s.name == name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Skill<Resolved>> {
        self.items.iter()
    }

    pub fn included(&self) -> impl Iterator<Item = &Skill<Resolved>> {
        self.items
            .iter()
            .filter(|s| s.state.decision == Decision::Included)
    }

    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ace_toml::AceToml;

    fn ace(skills: &[&str], inc: &[&str], exc: &[&str]) -> AceToml {
        AceToml {
            skills: skills.iter().map(|s| s.to_string()).collect(),
            include_skills: inc.iter().map(|s| s.to_string()).collect(),
            exclude_skills: exc.iter().map(|s| s.to_string()).collect(),
            ..AceToml::default()
        }
    }

    fn tree(project: AceToml) -> Tree {
        Tree {
            ace_user: AceToml::default(),
            ace_project: project,
            ace_local: AceToml::default(),
            school_backend: None,
            school_toml: None,
            school_paths: None,
        }
    }

    fn discovered(name: &str, tier: Tier) -> DiscoveredSkill {
        DiscoveredSkill {
            name: name.to_string(),
            path: PathBuf::from(format!("/school/{name}")),
            tier,
        }
    }

    #[test]
    fn resolve_preserves_path_and_tier() {
        let s = Skills::<Discovered>::from_discovered(&[
            discovered("a", Tier::Curated),
            discovered("b", Tier::Experimental),
        ]);
        let resolved = s.resolve(&tree(AceToml::default()));

        let a = resolved.find("a").expect("a");
        assert_eq!(a.path, PathBuf::from("/school/a"));
        assert_eq!(a.tier, Tier::Curated);
        assert_eq!(a.state.decision, Decision::Included); // implicit base
        assert_eq!(a.state.trace.len(), 1);

        let b = resolved.find("b").expect("b");
        assert_eq!(b.tier, Tier::Experimental);
    }

    #[test]
    fn included_filters_excluded() {
        let s = Skills::<Discovered>::from_discovered(&[
            discovered("a", Tier::Curated),
            discovered("b", Tier::Curated),
        ]);
        let resolved = s.resolve(&tree(ace(&["a"], &[], &[])));

        let included: Vec<&str> = resolved.included().map(|s| s.name.as_str()).collect();
        assert_eq!(included, vec!["a"]);

        // Both still iterable; only `b` is excluded.
        assert_eq!(resolved.iter().count(), 2);
    }

    #[test]
    fn diagnostics_carry_unknown_patterns() {
        let s = Skills::<Discovered>::from_discovered(&[discovered("a", Tier::Curated)]);
        let resolved = s.resolve(&tree(ace(&["nonexistent"], &[], &[])));

        let unk = &resolved.diagnostics().unknown_patterns;
        assert_eq!(unk.len(), 1);
        assert_eq!(unk[0].pattern, "nonexistent");
    }

    #[test]
    fn filter_tiers_keeps_only_allowed() {
        let s = Skills::<Discovered>::from_discovered(&[
            discovered("cur", Tier::Curated),
            discovered("exp", Tier::Experimental),
            discovered("sys", Tier::System),
        ]);
        let filtered = s.filter_tiers(&[Tier::Curated]);
        let names: Vec<&str> = filtered.names().collect();
        assert_eq!(names, vec!["cur"]);
    }

    #[test]
    fn matching_glob_after_filter() {
        let s = Skills::<Discovered>::from_discovered(&[
            discovered("a-cur", Tier::Curated),
            discovered("a-exp", Tier::Experimental),
            discovered("b-cur", Tier::Curated),
        ]);
        let filtered = s.filter_tiers(&[Tier::Curated]);
        let mut matched = filtered.matching("*-cur");
        matched.sort();
        assert_eq!(matched, vec!["a-cur", "b-cur"]);
    }
}
