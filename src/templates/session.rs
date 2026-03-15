use std::path::Path;

use crate::templates::builtins;
use crate::state::actions::update::{ChangeKind, SkillChange};

/// Build the session prompt from built-in + school + project layers.
pub fn build_session_prompt(
    school_name: &str,
    school_session_prompt: &str,
    project_session_prompt: &str,
    skills_dir: &Path,
    changes: &[SkillChange],
    school_cache: Option<&Path>,
) -> String {
    let mut parts = Vec::new();

    let builtin = format!("School: {school_name}\n\n{}", builtins::SESSION);
    parts.push(builtin);

    if !school_session_prompt.is_empty() {
        parts.push(school_session_prompt.to_string());
    }

    if !project_session_prompt.is_empty() {
        parts.push(project_session_prompt.to_string());
    }

    if !changes.is_empty() {
        parts.push(format_change_summary(changes));
    }

    if let Some(cache) = school_cache {
        parts.push(format!("School cache: {}", cache.display()));
    }

    let previous_skills = skills_dir.join("previous-skills");
    if previous_skills.exists() {
        let skills_dir_name = skills_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".claude");
        let tpl = super::Template::parse(builtins::PREVIOUS_SKILLS);
        let vals = std::collections::HashMap::from([
            ("skills_dir".to_string(), skills_dir_name.to_string()),
        ]);
        parts.push(tpl.substitute(&vals));
    }

    parts.join("\n\n")
}

fn format_change_summary(changes: &[SkillChange]) -> String {
    let mut added: Vec<&str> = Vec::new();
    let mut modified: Vec<&str> = Vec::new();
    let mut removed: Vec<&str> = Vec::new();

    for c in changes {
        match c.kind {
            ChangeKind::Added => added.push(&c.name),
            ChangeKind::Modified => modified.push(&c.name),
            ChangeKind::Removed => removed.push(&c.name),
        }
    }

    let mut lines = vec![builtins::CHANGES_HEADER.to_string()];
    for (label, names) in [("Added", added), ("Updated", modified), ("Removed", removed)] {
        for name in names {
            lines.push(format!("- {label}: `{name}`"));
        }
    }
    lines.push(format!("\n{}", builtins::CHANGES_FOOTER));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nonexistent_dir() -> std::path::PathBuf {
        std::path::PathBuf::from("/tmp/ace-test-prompt-nonexistent")
    }

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(name);
            let _ = std::fs::remove_dir_all(&path);
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn builtin_includes_session_base() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", "", "", &dir, &[], None);
        assert!(prompt.contains("School: Acme"));
        assert!(prompt.contains("ACE (AI Coding Environment)"));
        assert!(prompt.contains("propose changes back to the school repo"));
    }

    #[test]
    fn school_and_project_prompts() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", "Use Rust.", "PostgreSQL project.", &dir, &[], None);
        assert!(prompt.contains("Use Rust."));
        assert!(prompt.contains("PostgreSQL project."));
        let school_pos = prompt.find("Use Rust.").expect("school prompt present");
        let project_pos = prompt.find("PostgreSQL project.").expect("project prompt present");
        assert!(school_pos < project_pos, "school before project");
    }

    #[test]
    fn skips_empty_layers() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", "", "Only project.", &dir, &[], None);
        assert!(!prompt.contains("\n\n\n"), "no triple newlines from skipped school prompt");
        assert!(prompt.contains("Only project."));
    }

    #[test]
    fn injects_previous_skills_guidance() {
        let fix = TempDir::new("ace-test-prompt-previous");
        let prev = fix.path().join("previous-skills");
        std::fs::create_dir_all(&prev).expect("create previous-skills dir");

        let prompt = build_session_prompt("Acme", "", "", fix.path(), &[], None);
        assert!(prompt.contains("unconsolidated skills"));
        assert!(prompt.contains("previous-skills"));
    }

    #[test]
    fn previous_skills_uses_skills_dir_name() {
        let fix = TempDir::new("ace-test-prompt-opencode");
        let skills = fix.path().join(".opencode").join("previous-skills");
        std::fs::create_dir_all(&skills).expect("create previous-skills dir");

        let skills_dir = fix.path().join(".opencode");
        let prompt = build_session_prompt("Acme", "", "", &skills_dir, &[], None);
        assert!(prompt.contains(".opencode/previous-skills/"), "should use .opencode dir name");
        assert!(!prompt.contains(".claude/previous-skills/"), "should not contain .claude");
    }

    #[test]
    fn no_previous_skills_no_injection() {
        let fix = TempDir::new("ace-test-prompt-no-previous");
        std::fs::create_dir_all(fix.path().join("skills")).expect("create skills dir");

        let prompt = build_session_prompt("Acme", "", "", fix.path(), &[], None);
        assert!(!prompt.contains("unconsolidated"));
    }

    #[test]
    fn injects_change_summary() {
        let dir = nonexistent_dir();
        let changes = vec![
            SkillChange { name: "new-skill".into(), kind: ChangeKind::Added },
            SkillChange { name: "existing".into(), kind: ChangeKind::Modified },
            SkillChange { name: "old-skill".into(), kind: ChangeKind::Removed },
        ];

        let prompt = build_session_prompt("Acme", "", "", &dir, &changes, None);
        assert!(prompt.contains("School skills were updated"));
        assert!(prompt.contains("- Added: `new-skill`"));
        assert!(prompt.contains("- Updated: `existing`"));
        assert!(prompt.contains("- Removed: `old-skill`"));
    }

    #[test]
    fn no_changes_no_summary() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", "", "", &dir, &[], None);
        assert!(!prompt.contains("updated since your last session"));
    }

    #[test]
    fn injects_school_cache_path() {
        let dir = nonexistent_dir();
        let cache = Path::new("/home/user/.cache/ace/repos/org/school");
        let prompt = build_session_prompt("Acme", "", "", &dir, &[], Some(cache));
        assert!(prompt.contains("School cache: /home/user/.cache/ace/repos/org/school"));
    }

    #[test]
    fn no_cache_no_injection() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", "", "", &dir, &[], None);
        assert!(!prompt.contains("School cache:"));
    }
}
