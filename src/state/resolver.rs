//! Skill resolution: turns `(skills, include_skills, exclude_skills)` across
//! three config scopes into a structured trace per discovered skill.
//!
//! Pure logic. The trace drives `ace skills` (provenance listing) and
//! `ace explain <name>` (full chain).

// Public API is wired into production paths in step 3 (link rewrite) + step 4 (CLI).
// Module-level allow keeps the staged-integration warnings off the build until then;
// removed when callers land.
#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::config::ace_toml::AceToml;
use crate::glob::glob_match;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub skills: Vec<ResolvedSkill>,
    pub unknown_patterns: Vec<UnknownPattern>,
    pub collisions: Vec<Collision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSkill {
    pub name: String,
    pub decision: Decision,
    pub trace: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub scope: Scope,
    pub field: Field,
    pub pattern: String,
    pub op: Op,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownPattern {
    pub scope: Scope,
    pub field: Field,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collision {
    pub skill: String,
    pub scope: Scope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Included,
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    SetBase,
    Added,
    Removed,
    ReAdded,
}

/// Provenance scope for a trace entry. Distinct from `config::Scope` —
/// adds `Implicit` for the synthetic "no `skills` filter set" baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Implicit,
    User,
    Project,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Skills,
    IncludeSkills,
    ExcludeSkills,
}

pub fn resolve(
    discovered: &[String],
    user: &AceToml,
    project: &AceToml,
    local: &AceToml,
) -> Resolution {
    let mut state: BTreeMap<String, ResolvedSkill> = discovered
        .iter()
        .map(|name| {
            (
                name.clone(),
                ResolvedSkill {
                    name: name.clone(),
                    decision: Decision::Excluded,
                    trace: Vec::new(),
                },
            )
        })
        .collect();
    let mut unknown_patterns: Vec<UnknownPattern> = Vec::new();

    apply_base(&mut state, &mut unknown_patterns, user, project, local);
    apply_phase(
        &mut state,
        &mut unknown_patterns,
        Phase::Exclude,
        scoped(user, project, local, |a| &a.exclude_skills),
    );
    apply_phase(
        &mut state,
        &mut unknown_patterns,
        Phase::Include,
        scoped(user, project, local, |a| &a.include_skills),
    );

    let collisions = detect_collisions(&state);

    Resolution {
        skills: state.into_values().collect(),
        unknown_patterns,
        collisions,
    }
}

fn scoped<'a, F>(
    user: &'a AceToml,
    project: &'a AceToml,
    local: &'a AceToml,
    pick: F,
) -> Vec<(Scope, &'a [String])>
where
    F: Fn(&'a AceToml) -> &'a Vec<String>,
{
    vec![
        (Scope::User, pick(user).as_slice()),
        (Scope::Project, pick(project).as_slice()),
        (Scope::Local, pick(local).as_slice()),
    ]
}

fn apply_base(
    state: &mut BTreeMap<String, ResolvedSkill>,
    unknown: &mut Vec<UnknownPattern>,
    user: &AceToml,
    project: &AceToml,
    local: &AceToml,
) {
    let winner = if !local.skills.is_empty() {
        Some((Scope::Local, &local.skills))
    } else if !project.skills.is_empty() {
        Some((Scope::Project, &project.skills))
    } else if !user.skills.is_empty() {
        Some((Scope::User, &user.skills))
    } else {
        None
    };

    let Some((scope, patterns)) = winner else {
        for skill in state.values_mut() {
            skill.trace.push(Entry {
                scope: Scope::Implicit,
                field: Field::Skills,
                pattern: "*".to_string(),
                op: Op::SetBase,
            });
            skill.decision = Decision::Included;
        }
        return;
    };

    for pattern in patterns {
        let mut matched = false;
        for skill in state.values_mut() {
            if !glob_match(pattern, &skill.name) {
                continue;
            }
            matched = true;
            skill.trace.push(Entry {
                scope,
                field: Field::Skills,
                pattern: pattern.clone(),
                op: Op::SetBase,
            });
            skill.decision = Decision::Included;
        }
        if !matched {
            unknown.push(UnknownPattern {
                scope,
                field: Field::Skills,
                pattern: pattern.clone(),
            });
        }
    }
}

/// Restricted phase indicator — `Skills` is intentionally absent. The base
/// phase is handled by `apply_base`; everything else is exclude or include.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Exclude,
    Include,
}

impl Phase {
    fn field(self) -> Field {
        match self {
            Phase::Exclude => Field::ExcludeSkills,
            Phase::Include => Field::IncludeSkills,
        }
    }

    fn decision(self) -> Decision {
        match self {
            Phase::Exclude => Decision::Excluded,
            Phase::Include => Decision::Included,
        }
    }

    /// Op to record when a pattern matches this skill, or `None` to skip
    /// because the skill is already in the target decision (no state change).
    fn op_for(self, skill: &ResolvedSkill) -> Option<Op> {
        match (self, skill.decision) {
            (Phase::Exclude, Decision::Excluded) => None,
            (Phase::Exclude, Decision::Included) => Some(Op::Removed),
            (Phase::Include, Decision::Included) => Some(Op::Added),
            (Phase::Include, Decision::Excluded) => {
                let was_removed = skill.trace.iter().any(|e| e.op == Op::Removed);
                Some(if was_removed { Op::ReAdded } else { Op::Added })
            }
        }
    }
}

fn apply_phase(
    state: &mut BTreeMap<String, ResolvedSkill>,
    unknown: &mut Vec<UnknownPattern>,
    phase: Phase,
    sources: Vec<(Scope, &[String])>,
) {
    let field = phase.field();
    for (scope, patterns) in sources {
        for pattern in patterns {
            let mut matched = false;
            for skill in state.values_mut() {
                if !glob_match(pattern, &skill.name) {
                    continue;
                }
                matched = true;
                let Some(op) = phase.op_for(skill) else {
                    continue;
                };
                skill.trace.push(Entry {
                    scope,
                    field,
                    pattern: pattern.clone(),
                    op,
                });
                skill.decision = phase.decision();
            }
            if !matched {
                unknown.push(UnknownPattern {
                    scope,
                    field,
                    pattern: pattern.clone(),
                });
            }
        }
    }
}

fn detect_collisions(state: &BTreeMap<String, ResolvedSkill>) -> Vec<Collision> {
    let mut collisions = Vec::new();
    for skill in state.values() {
        for target in [Scope::User, Scope::Project, Scope::Local] {
            let has_remove = skill
                .trace
                .iter()
                .any(|e| e.scope == target && e.field == Field::ExcludeSkills);
            let has_add = skill
                .trace
                .iter()
                .any(|e| e.scope == target && e.field == Field::IncludeSkills);
            if has_remove && has_add {
                collisions.push(Collision {
                    skill: skill.name.clone(),
                    scope: target,
                });
            }
        }
    }
    collisions
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

    fn names() -> Vec<String> {
        ["a", "b", "rust-coding", "rust-fmt", "issue-tracker"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn included(r: &Resolution) -> Vec<&str> {
        r.skills
            .iter()
            .filter(|s| s.decision == Decision::Included)
            .map(|s| s.name.as_str())
            .collect()
    }

    fn excluded(r: &Resolution) -> Vec<&str> {
        r.skills
            .iter()
            .filter(|s| s.decision == Decision::Excluded)
            .map(|s| s.name.as_str())
            .collect()
    }

    fn find<'a>(r: &'a Resolution, name: &str) -> &'a ResolvedSkill {
        r.skills
            .iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("skill {name} missing from resolution"))
    }

    // base: empty everywhere = all included with implicit scope.

    #[test]
    fn all_empty_includes_everything_with_implicit_base() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        assert_eq!(included(&r), vec!["a", "b", "issue-tracker", "rust-coding", "rust-fmt"]);
        assert!(excluded(&r).is_empty());
        let s = find(&r, "a");
        assert_eq!(s.trace.len(), 1);
        assert_eq!(s.trace[0].scope, Scope::Implicit);
        assert_eq!(s.trace[0].field, Field::Skills);
        assert_eq!(s.trace[0].op, Op::SetBase);
    }

    // skills filter: only matching names in base.

    #[test]
    fn project_skills_filter_narrows_base() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&["rust-*"], &[], &[]),
            &AceToml::default(),
        );
        assert_eq!(included(&r), vec!["rust-coding", "rust-fmt"]);
        let rc = find(&r, "rust-coding");
        assert_eq!(rc.trace[0].scope, Scope::Project);
        assert_eq!(rc.trace[0].pattern, "rust-*");
        assert_eq!(rc.trace[0].op, Op::SetBase);
        // unmatched skills have no trace and are Excluded
        let a = find(&r, "a");
        assert!(a.trace.is_empty());
        assert_eq!(a.decision, Decision::Excluded);
    }

    #[test]
    fn local_skills_overrides_project_skills() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&["rust-*"], &[], &[]),
            &ace(&["a"], &[], &[]),
        );
        assert_eq!(included(&r), vec!["a"]);
        let a = find(&r, "a");
        assert_eq!(a.trace[0].scope, Scope::Local);
    }

    // include_skills: union across scopes; adds to base.

    #[test]
    fn user_include_skills_adds_to_project_base() {
        let r = resolve(
            &names(),
            &ace(&[], &["issue-*"], &[]),
            &ace(&["rust-*"], &[], &[]),
            &AceToml::default(),
        );
        assert_eq!(included(&r), vec!["issue-tracker", "rust-coding", "rust-fmt"]);
        let it = find(&r, "issue-tracker");
        assert_eq!(it.trace.len(), 1);
        assert_eq!(it.trace[0].scope, Scope::User);
        assert_eq!(it.trace[0].field, Field::IncludeSkills);
        assert_eq!(it.trace[0].op, Op::Added);
    }

    // exclude_skills: removes from base.

    #[test]
    fn local_exclude_skills_removes_from_base() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&["rust-*"], &[], &[]),
            &ace(&[], &[], &["rust-fmt"]),
        );
        assert_eq!(included(&r), vec!["rust-coding"]);
        let rf = find(&r, "rust-fmt");
        assert_eq!(rf.decision, Decision::Excluded);
        assert_eq!(rf.trace.len(), 2);
        assert_eq!(rf.trace[0].op, Op::SetBase);
        assert_eq!(rf.trace[1].op, Op::Removed);
        assert_eq!(rf.trace[1].scope, Scope::Local);
        assert_eq!(rf.trace[1].field, Field::ExcludeSkills);
    }

    // include re-adds excluded -> ReAdded op.

    #[test]
    fn include_readds_excluded() {
        let r = resolve(
            &names(),
            &ace(&[], &["rust-fmt"], &[]),
            &ace(&["rust-*"], &[], &["rust-fmt"]),
            &AceToml::default(),
        );
        let rf = find(&r, "rust-fmt");
        assert_eq!(rf.decision, Decision::Included);
        assert_eq!(rf.trace.len(), 3);
        assert_eq!(rf.trace[0].op, Op::SetBase);
        assert_eq!(rf.trace[1].op, Op::Removed);
        assert_eq!(rf.trace[2].op, Op::ReAdded);
        assert_eq!(rf.trace[2].scope, Scope::User);
    }

    // same-scope collision: include + exclude in same file matching same skill.

    #[test]
    fn same_scope_collision_reported() {
        let r = resolve(
            &names(),
            &ace(&[], &["a"], &["a"]),
            &AceToml::default(),
            &AceToml::default(),
        );
        assert_eq!(r.collisions.len(), 1);
        assert_eq!(r.collisions[0].skill, "a");
        assert_eq!(r.collisions[0].scope, Scope::User);
    }

    #[test]
    fn cross_scope_include_exclude_is_not_a_collision() {
        let r = resolve(
            &names(),
            &ace(&[], &["a"], &[]),
            &AceToml::default(),
            &ace(&[], &[], &["a"]),
        );
        assert!(r.collisions.is_empty());
    }

    // unknown patterns: pattern matches nothing.

    #[test]
    fn unknown_pattern_surfaced() {
        let r = resolve(
            &names(),
            &ace(&[], &["typo-*"], &[]),
            &AceToml::default(),
            &AceToml::default(),
        );
        assert_eq!(r.unknown_patterns.len(), 1);
        assert_eq!(r.unknown_patterns[0].pattern, "typo-*");
        assert_eq!(r.unknown_patterns[0].scope, Scope::User);
        assert_eq!(r.unknown_patterns[0].field, Field::IncludeSkills);
    }

    // glob expansion: one pattern, multiple matches.

    #[test]
    fn glob_pattern_matches_multiple_names() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&["rust-*"], &[], &[]),
            &AceToml::default(),
        );
        let rc = find(&r, "rust-coding");
        let rf = find(&r, "rust-fmt");
        assert_eq!(rc.trace[0].pattern, "rust-*");
        assert_eq!(rf.trace[0].pattern, "rust-*");
    }

    // skills sorted by name in output.

    #[test]
    fn output_sorted_by_name() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let names: Vec<&str> = r.skills.iter().map(|s| s.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    // include matching an already-base-included skill records an extra Added
    // entry — every contribution is preserved in the trace for `ace explain`.

    #[test]
    fn include_on_already_included_skill_adds_extra_entry() {
        let r = resolve(
            &names(),
            &ace(&[], &["a"], &[]),
            &ace(&["a"], &[], &[]),
            &AceToml::default(),
        );
        let a = find(&r, "a");
        assert_eq!(a.decision, Decision::Included);
        assert_eq!(a.trace.len(), 2);
        assert_eq!(a.trace[0].op, Op::SetBase);
        assert_eq!(a.trace[0].scope, Scope::Project);
        assert_eq!(a.trace[1].op, Op::Added);
        assert_eq!(a.trace[1].scope, Scope::User);
    }

    // exact-name matching also works (not just globs).

    #[test]
    fn exact_name_pattern_matches() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&["rust-coding"], &[], &[]),
            &AceToml::default(),
        );
        assert_eq!(included(&r), vec!["rust-coding"]);
    }
}
