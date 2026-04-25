//! Build and render the `ace skills` listing.
//!
//! Pure: takes a `Resolution` (from `state::resolver`) plus a name→tier map
//! (from `state::discover`) and produces row data. Renderers turn rows into
//! tab-separated tables or bare-name listings. The action wrapper does I/O.

use std::collections::HashMap;
use std::fmt::Write;

use crate::state::discover::Tier;
use crate::state::resolver::{Decision, Entry, Field, Resolution, ResolvedSkill, Scope};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillRow {
    pub name: String,
    pub tier: Option<Tier>,
    pub status: Status,
    /// Scope+field of the last contribution (None only when no trace exists,
    /// which doesn't happen in practice — every skill gets at least an Implicit base).
    pub source: Option<(Scope, Field)>,
    pub reason: String,
}

/// Build rows from a resolution. Default hides excluded; `show_excluded`
/// includes them. Output sorted by name.
pub fn build_rows(
    resolution: &Resolution,
    tiers: &HashMap<String, Tier>,
    show_excluded: bool,
) -> Vec<SkillRow> {
    let mut rows: Vec<SkillRow> = resolution
        .skills
        .iter()
        .filter(|s| show_excluded || s.decision == Decision::Included)
        .map(|s| SkillRow {
            name: s.name.clone(),
            tier: tiers.get(&s.name).copied(),
            status: status_of(s),
            source: s.trace.last().map(|e| (e.scope, e.field)),
            reason: reason_for(s),
        })
        .collect();
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    rows
}

fn status_of(s: &ResolvedSkill) -> Status {
    match s.decision {
        Decision::Included => Status::Active,
        Decision::Excluded => Status::Excluded,
    }
}

/// Human-readable summary of the last trace contribution.
fn reason_for(s: &ResolvedSkill) -> String {
    let Some(last) = s.trace.last() else {
        return String::new();
    };
    format_entry(last)
}

fn format_entry(e: &Entry) -> String {
    let scope = scope_label(e.scope);
    let field = field_label(e.field);
    match e.scope {
        Scope::Implicit => "implicit base (no skills filter)".to_string(),
        _ => format!("{scope}: {field} \"{}\"", e.pattern),
    }
}

fn scope_label(s: Scope) -> &'static str {
    match s {
        Scope::Implicit => "implicit",
        Scope::User => "user",
        Scope::Project => "project",
        Scope::Local => "local",
    }
}

fn field_label(f: Field) -> &'static str {
    match f {
        Field::Skills => "skills",
        Field::IncludeSkills => "include_skills",
        Field::ExcludeSkills => "exclude_skills",
    }
}

fn tier_label(t: Option<Tier>) -> &'static str {
    match t {
        Some(Tier::Curated) => "curated",
        Some(Tier::Experimental) => "experimental",
        Some(Tier::System) => "system",
        None => "-",
    }
}

fn status_label(s: Status) -> &'static str {
    match s {
        Status::Active => "active",
        Status::Excluded => "excluded",
    }
}

/// Tab-separated table with header. Matches `ace paths` style for machine parsing.
pub fn render_table(rows: &[SkillRow]) -> String {
    let mut out = String::from("NAME\tTIER\tSTATUS\tREASON\n");
    for r in rows {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}",
            r.name,
            tier_label(r.tier),
            status_label(r.status),
            r.reason,
        );
    }
    out
}

/// Bare names, one per line. Scriptable.
pub fn render_names(rows: &[SkillRow]) -> String {
    let mut out = String::new();
    for r in rows {
        out.push_str(&r.name);
        out.push('\n');
    }
    out
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
        ["a", "b", "rust-coding", "rust-fmt"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn all_curated() -> HashMap<String, Tier> {
        names().into_iter().map(|n| (n, Tier::Curated)).collect()
    }

    #[test]
    fn default_hides_excluded() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&["rust-*"], &[], &[]),
            &AceToml::default(),
        );
        let rows = build_rows(&r, &all_curated(), false);
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["rust-coding", "rust-fmt"]);
        assert!(rows.iter().all(|r| r.status == Status::Active));
    }

    #[test]
    fn show_all_includes_excluded() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&["rust-*"], &[], &[]),
            &AceToml::default(),
        );
        let rows = build_rows(&r, &all_curated(), true);
        assert_eq!(rows.len(), 4);
        let excluded: Vec<&str> = rows
            .iter()
            .filter(|r| r.status == Status::Excluded)
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(excluded, vec!["a", "b"]);
    }

    #[test]
    fn rows_sorted_by_name() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let rows = build_rows(&r, &all_curated(), false);
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn source_picks_last_contribution() {
        // Project sets base, user adds via include — last contribution is User/IncludeSkills.
        let r = resolve(
            &names(),
            &ace(&[], &["rust-*"], &[]),
            &ace(&["a"], &[], &[]),
            &AceToml::default(),
        );
        let rows = build_rows(&r, &all_curated(), false);
        let rc = rows.iter().find(|r| r.name == "rust-coding").expect("rust-coding row");
        assert_eq!(rc.source, Some((Scope::User, Field::IncludeSkills)));
        assert!(rc.reason.contains("user"));
        assert!(rc.reason.contains("include_skills"));
        assert!(rc.reason.contains("rust-*"));
    }

    #[test]
    fn implicit_base_reason_when_all_empty() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let rows = build_rows(&r, &all_curated(), false);
        let a = rows.iter().find(|r| r.name == "a").expect("a row");
        assert_eq!(a.source, Some((Scope::Implicit, Field::Skills)));
        assert!(a.reason.contains("implicit"));
    }

    #[test]
    fn tier_passthrough() {
        let mut tiers = HashMap::new();
        tiers.insert("a".to_string(), Tier::Curated);
        tiers.insert("b".to_string(), Tier::Experimental);
        tiers.insert("rust-coding".to_string(), Tier::System);
        // rust-fmt intentionally absent → tier should be None
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let rows = build_rows(&r, &tiers, false);
        let tier = |name: &str| {
            rows.iter().find(|r| r.name == name).expect("row").tier
        };
        assert_eq!(tier("a"), Some(Tier::Curated));
        assert_eq!(tier("b"), Some(Tier::Experimental));
        assert_eq!(tier("rust-coding"), Some(Tier::System));
        assert_eq!(tier("rust-fmt"), None);
    }

    #[test]
    fn excluded_row_reports_removal_reason() {
        let r = resolve(
            &names(),
            &AceToml::default(),
            &ace(&[], &[], &["a"]),
            &AceToml::default(),
        );
        let rows = build_rows(&r, &all_curated(), true);
        let a = rows.iter().find(|r| r.name == "a").expect("a row");
        assert_eq!(a.status, Status::Excluded);
        assert_eq!(a.source, Some((Scope::Project, Field::ExcludeSkills)));
        assert!(a.reason.contains("project"));
        assert!(a.reason.contains("exclude_skills"));
    }

    #[test]
    fn render_table_has_header_and_one_line_per_row() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let rows = build_rows(&r, &all_curated(), false);
        let out = render_table(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "NAME\tTIER\tSTATUS\tREASON");
        assert_eq!(lines.len(), 1 + rows.len());
        // every data line has 4 tab-separated fields
        for line in &lines[1..] {
            assert_eq!(line.split('\t').count(), 4, "line: {line:?}");
        }
    }

    #[test]
    fn render_names_one_per_line() {
        let r = resolve(&names(), &AceToml::default(), &AceToml::default(), &AceToml::default());
        let rows = build_rows(&r, &all_curated(), false);
        let out = render_names(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines, vec!["a", "b", "rust-coding", "rust-fmt"]);
    }

    #[test]
    fn render_table_empty_rows_just_header() {
        let out = render_table(&[]);
        assert_eq!(out, "NAME\tTIER\tSTATUS\tREASON\n");
    }
}
