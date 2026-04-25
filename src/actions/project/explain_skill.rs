//! Detailed explanation of a single skill's resolution + provenance.
//!
//! Pure: takes a `Resolution`, a name→tier map, and a target name; returns
//! the full trace formatted for human reading. Unknown names produce a list
//! of near-matches (simple substring / prefix overlap, no Levenshtein).

use std::collections::HashMap;

use crate::state::discover::Tier;
use crate::state::resolver::{Decision, Entry, Field, Op, Resolution, ResolvedSkill, Scope};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainOutput {
    pub name: String,
    pub tier: Option<Tier>,
    pub status: Status,
    pub trace_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Excluded,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ExplainError {
    #[error("unknown skill `{name}`{}", format_near(near))]
    NotFound { name: String, near: Vec<String> },
}

fn format_near(near: &[String]) -> String {
    if near.is_empty() {
        String::new()
    } else {
        format!(" — did you mean: {}", near.join(", "))
    }
}

pub fn build(
    resolution: &Resolution,
    tiers: &HashMap<String, Tier>,
    name: &str,
) -> Result<ExplainOutput, ExplainError> {
    let Some(skill) = resolution.skills.iter().find(|s| s.name == name) else {
        let near = near_matches(name, &resolution.skills);
        return Err(ExplainError::NotFound { name: name.to_string(), near });
    };

    Ok(ExplainOutput {
        name: skill.name.clone(),
        tier: tiers.get(&skill.name).copied(),
        status: status_of(skill),
        trace_lines: skill.trace.iter().map(format_entry).collect(),
    })
}

fn status_of(s: &ResolvedSkill) -> Status {
    match s.decision {
        Decision::Included => Status::Active,
        Decision::Excluded => Status::Excluded,
    }
}

fn format_entry(e: &Entry) -> String {
    let op = match e.op {
        Op::SetBase => "base",
        Op::Added => "added",
        Op::Removed => "removed",
        Op::ReAdded => "re-added",
    };
    let scope = match e.scope {
        Scope::Implicit => "implicit",
        Scope::User => "user",
        Scope::Project => "project",
        Scope::Local => "local",
    };
    let field = match e.field {
        Field::Skills => "skills",
        Field::IncludeSkills => "include_skills",
        Field::ExcludeSkills => "exclude_skills",
    };
    format!("{op:>9}  {scope}: {field} \"{}\"", e.pattern)
}

/// Up to 5 names with at least 3-char overlap with the query. Cheap heuristic;
/// good enough for "did you mean" prompts without pulling in a fuzzy crate.
fn near_matches(query: &str, skills: &[ResolvedSkill]) -> Vec<String> {
    let q = query.to_lowercase();
    let mut scored: Vec<(usize, &str)> = skills
        .iter()
        .filter_map(|s| {
            let name = s.name.as_str();
            let lower = name.to_lowercase();
            let score = overlap_score(&q, &lower);
            (score >= 3).then_some((score, name))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    scored.into_iter().take(5).map(|(_, n)| n.to_string()).collect()
}

/// Length of longest contiguous substring of `q` appearing in `name`.
fn overlap_score(q: &str, name: &str) -> usize {
    let qb = q.as_bytes();
    let nb = name.as_bytes();
    if qb.is_empty() || nb.is_empty() {
        return 0;
    }
    let mut best = 0;
    for start in 0..qb.len() {
        for end in (start + 1..=qb.len()).rev() {
            if end - start <= best {
                break;
            }
            let sub = &q[start..end];
            if name.contains(sub) {
                best = end - start;
                break;
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ace_toml::AceToml;
    use crate::state::resolver::resolve;

    fn ace(skills: &[&str], inc: &[&str], exc: &[&str]) -> AceToml {
        AceToml {
            skills: skills.iter().map(|s| s.to_string()).collect(),
            include_skills: inc.iter().map(|s| s.to_string()).collect(),
            exclude_skills: exc.iter().map(|s| s.to_string()).collect(),
            ..AceToml::default()
        }
    }

    fn names() -> Vec<String> {
        ["a", "b", "rust-coding", "rust-fmt", "issue-tracker"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn tiers() -> HashMap<String, Tier> {
        let mut m = HashMap::new();
        for n in names() {
            m.insert(n, Tier::Curated);
        }
        m
    }

    #[test]
    fn explains_active_skill_with_base_only() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let out = build(&r, &tiers(), "rust-coding").expect("known skill");
        assert_eq!(out.name, "rust-coding");
        assert_eq!(out.status, Status::Active);
        assert_eq!(out.tier, Some(Tier::Curated));
        assert_eq!(out.trace_lines.len(), 1);
        assert!(out.trace_lines[0].contains("base"));
        assert!(out.trace_lines[0].contains("implicit"));
    }

    #[test]
    fn explains_excluded_skill() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&[], &[], &["rust-fmt"]),
            &AceToml::default(),
        );
        let out = build(&r, &tiers(), "rust-fmt").expect("known skill");
        assert_eq!(out.status, Status::Excluded);
        assert_eq!(out.trace_lines.len(), 2);
        assert!(out.trace_lines[0].contains("base"));
        assert!(out.trace_lines[1].contains("removed"));
        assert!(out.trace_lines[1].contains("project"));
        assert!(out.trace_lines[1].contains("exclude_skills"));
        assert!(out.trace_lines[1].contains("rust-fmt"));
    }

    #[test]
    fn explains_readded_skill_shows_full_chain() {
        let r = resolve(
            &names(),
            &ace(&[], &["rust-fmt"], &[]),
            &ace(&["rust-*"], &[], &["rust-fmt"]),
            &AceToml::default(),
        );
        let out = build(&r, &tiers(), "rust-fmt").expect("known skill");
        assert_eq!(out.status, Status::Active);
        assert_eq!(out.trace_lines.len(), 3);
        assert!(out.trace_lines[0].contains("base"));
        assert!(out.trace_lines[1].contains("removed"));
        assert!(out.trace_lines[2].contains("re-added"));
    }

    #[test]
    fn unknown_skill_returns_not_found_with_near_matches() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let err = build(&r, &tiers(), "rust-cod").unwrap_err();
        match err {
            ExplainError::NotFound { name, near } => {
                assert_eq!(name, "rust-cod");
                assert!(near.contains(&"rust-coding".to_string()), "got: {near:?}");
            }
        }
    }

    #[test]
    fn unknown_skill_with_no_overlap_returns_empty_near() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let err = build(&r, &tiers(), "xz").unwrap_err();
        match err {
            ExplainError::NotFound { near, .. } => assert!(near.is_empty(), "got: {near:?}"),
        }
    }

    #[test]
    fn near_matches_capped_at_five() {
        let many: Vec<String> = (0..20).map(|i| format!("skill-{i:02}")).collect();
        let r = resolve(&many, &AceToml::default(), &AceToml::default(), &AceToml::default());
        let mut t = HashMap::new();
        for n in &many {
            t.insert(n.clone(), Tier::Curated);
        }
        let err = build(&r, &t, "skill-").unwrap_err();
        match err {
            ExplainError::NotFound { near, .. } => {
                assert_eq!(near.len(), 5, "should cap near matches; got {}", near.len());
            }
        }
    }

    #[test]
    fn tier_passthrough_when_present() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let mut t = HashMap::new();
        t.insert("rust-coding".to_string(), Tier::System);
        let out = build(&r, &t, "rust-coding").expect("known skill");
        assert_eq!(out.tier, Some(Tier::System));
    }

    #[test]
    fn tier_none_when_absent_from_map() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let out = build(&r, &HashMap::new(), "rust-coding").expect("known skill");
        assert_eq!(out.tier, None);
    }
}
