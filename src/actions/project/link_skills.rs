//! Per-skill symlink reconciliation.
//!
//! Replaces the legacy whole-dir `<backend>/skills` symlink. The skills
//! directory becomes a real dir; each enabled skill gets its own symlink
//! pointing into the school clone. Re-runs reconcile in place: add, repoint,
//! remove ACE-managed links to match the desired set; warn on foreign entries.
//!
//! ACE-managed predicate: a symlink whose target resolves textually inside
//! the school clone's `skills/` subtree. No marker files.

// Wired into Link / Prepare in the next commit. Module-level allow keeps the
// staged-integration warnings quiet until then; removed when callers land.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesiredLink {
    pub name: String,
    pub target: PathBuf,
}

/// Snapshot of one entry currently inside `<backend>/skills/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentEntry {
    pub name: String,
    pub kind: EntryKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    /// Symlink whose target resolves inside `school_skills_root` — safe to manage.
    ManagedSymlink { target: PathBuf },
    /// Symlink with a target outside the school skills root — leave alone.
    ForeignSymlink { target: PathBuf },
    /// Real file or directory placed by the user — leave alone.
    ForeignEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkAction {
    Create { name: String, target: PathBuf },
    Repoint { name: String, target: PathBuf },
    Remove { name: String },
    SkipForeign { name: String, reason: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkPlan {
    pub actions: Vec<LinkAction>,
}

/// Compute the reconciliation plan. Pure: no I/O.
pub fn plan(desired: &[DesiredLink], current: &[CurrentEntry]) -> LinkPlan {
    let mut actions = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for want in desired {
        seen.insert(want.name.as_str());
        let existing = current.iter().find(|c| c.name == want.name);
        match existing {
            None => actions.push(LinkAction::Create {
                name: want.name.clone(),
                target: want.target.clone(),
            }),
            Some(entry) => match &entry.kind {
                EntryKind::ManagedSymlink { target } if target == &want.target => {} // up-to-date
                EntryKind::ManagedSymlink { .. } => actions.push(LinkAction::Repoint {
                    name: want.name.clone(),
                    target: want.target.clone(),
                }),
                EntryKind::ForeignSymlink { target } => actions.push(LinkAction::SkipForeign {
                    name: want.name.clone(),
                    reason: format!(
                        "symlink points outside school clone: {}",
                        target.display()
                    ),
                }),
                EntryKind::ForeignEntry => actions.push(LinkAction::SkipForeign {
                    name: want.name.clone(),
                    reason: "not managed by ace (file or directory exists)".to_string(),
                }),
            },
        }
    }

    for entry in current {
        if seen.contains(entry.name.as_str()) {
            continue;
        }
        if matches!(entry.kind, EntryKind::ManagedSymlink { .. }) {
            actions.push(LinkAction::Remove { name: entry.name.clone() });
        }
        // Foreign entries with no desired counterpart: leave alone, no warning needed.
    }

    LinkPlan { actions }
}

/// Classify a directory entry. Reads the symlink target if applicable;
/// pure given the input string slices (no further I/O).
pub fn classify(name: &str, kind_input: ClassifyInput, school_skills_root: &Path) -> CurrentEntry {
    let kind = match kind_input {
        ClassifyInput::Symlink(target) => {
            if target.starts_with(school_skills_root) {
                EntryKind::ManagedSymlink { target }
            } else {
                EntryKind::ForeignSymlink { target }
            }
        }
        ClassifyInput::Other => EntryKind::ForeignEntry,
    };
    CurrentEntry {
        name: name.to_string(),
        kind,
    }
}

/// Pulled out so `classify` stays pure. The I/O wrapper packages disk reads
/// into one of these variants.
pub enum ClassifyInput {
    Symlink(PathBuf),
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desired(pairs: &[(&str, &str)]) -> Vec<DesiredLink> {
        pairs
            .iter()
            .map(|(n, t)| DesiredLink {
                name: (*n).to_string(),
                target: PathBuf::from(*t),
            })
            .collect()
    }

    fn managed(name: &str, target: &str) -> CurrentEntry {
        CurrentEntry {
            name: name.to_string(),
            kind: EntryKind::ManagedSymlink {
                target: PathBuf::from(target),
            },
        }
    }

    fn foreign_link(name: &str, target: &str) -> CurrentEntry {
        CurrentEntry {
            name: name.to_string(),
            kind: EntryKind::ForeignSymlink {
                target: PathBuf::from(target),
            },
        }
    }

    fn foreign_entry(name: &str) -> CurrentEntry {
        CurrentEntry {
            name: name.to_string(),
            kind: EntryKind::ForeignEntry,
        }
    }

    #[test]
    fn empty_dir_creates_all_desired() {
        let p = plan(&desired(&[("a", "/sch/a"), ("b", "/sch/b")]), &[]);
        assert_eq!(
            p.actions,
            vec![
                LinkAction::Create { name: "a".into(), target: "/sch/a".into() },
                LinkAction::Create { name: "b".into(), target: "/sch/b".into() },
            ]
        );
    }

    #[test]
    fn correct_managed_link_is_left_alone() {
        let p = plan(
            &desired(&[("a", "/sch/a")]),
            &[managed("a", "/sch/a")],
        );
        assert!(p.actions.is_empty());
    }

    #[test]
    fn stale_managed_link_is_repointed() {
        let p = plan(
            &desired(&[("a", "/sch/a-new")]),
            &[managed("a", "/sch/a-old")],
        );
        assert_eq!(
            p.actions,
            vec![LinkAction::Repoint { name: "a".into(), target: "/sch/a-new".into() }]
        );
    }

    #[test]
    fn orphaned_managed_link_is_removed() {
        let p = plan(
            &desired(&[("b", "/sch/b")]),
            &[managed("a", "/sch/a"), managed("b", "/sch/b")],
        );
        assert_eq!(p.actions, vec![LinkAction::Remove { name: "a".into() }]);
    }

    #[test]
    fn foreign_symlink_is_skipped_with_reason() {
        let p = plan(
            &desired(&[("a", "/sch/a")]),
            &[foreign_link("a", "/elsewhere")],
        );
        assert_eq!(p.actions.len(), 1);
        assert!(matches!(p.actions[0], LinkAction::SkipForeign { .. }));
        if let LinkAction::SkipForeign { reason, .. } = &p.actions[0] {
            assert!(reason.contains("/elsewhere"));
        }
    }

    #[test]
    fn foreign_real_entry_is_skipped() {
        let p = plan(
            &desired(&[("a", "/sch/a")]),
            &[foreign_entry("a")],
        );
        assert_eq!(p.actions.len(), 1);
        assert!(matches!(p.actions[0], LinkAction::SkipForeign { .. }));
    }

    #[test]
    fn foreign_orphan_is_left_alone() {
        // User dropped a real dir for a skill we don't link — no action, no warn.
        let p = plan(&desired(&[]), &[foreign_entry("user-stuff")]);
        assert!(p.actions.is_empty());
    }

    #[test]
    fn classify_managed_when_target_inside_root() {
        let entry = classify(
            "a",
            ClassifyInput::Symlink(PathBuf::from("/sch/skills/a")),
            Path::new("/sch/skills"),
        );
        assert_eq!(
            entry.kind,
            EntryKind::ManagedSymlink { target: PathBuf::from("/sch/skills/a") }
        );
    }

    #[test]
    fn classify_foreign_when_target_outside_root() {
        let entry = classify(
            "a",
            ClassifyInput::Symlink(PathBuf::from("/elsewhere/a")),
            Path::new("/sch/skills"),
        );
        assert!(matches!(entry.kind, EntryKind::ForeignSymlink { .. }));
    }

    #[test]
    fn classify_other_is_foreign_entry() {
        let entry = classify("a", ClassifyInput::Other, Path::new("/sch/skills"));
        assert_eq!(entry.kind, EntryKind::ForeignEntry);
    }

    #[test]
    fn mixed_scenario() {
        // desired: a (new), b (correct), c (repoint)
        // current: b (correct), c (stale), d (orphan-managed), foo (orphan-foreign)
        let p = plan(
            &desired(&[("a", "/sch/a"), ("b", "/sch/b"), ("c", "/sch/c-new")]),
            &[
                managed("b", "/sch/b"),
                managed("c", "/sch/c-old"),
                managed("d", "/sch/d"),
                foreign_entry("foo"),
            ],
        );
        assert_eq!(
            p.actions,
            vec![
                LinkAction::Create { name: "a".into(), target: "/sch/a".into() },
                LinkAction::Repoint { name: "c".into(), target: "/sch/c-new".into() },
                LinkAction::Remove { name: "d".into() },
            ]
        );
    }
}
