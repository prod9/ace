use std::path::Path;

use super::actions::update::{ChangeKind, SkillChange};

const SCHOOL_EDITABLE: &str = include_str!("prompts/school_editable.txt");
const PREVIOUS_SKILLS: &str = include_str!("prompts/previous_skills.txt");
const CHANGES_HEADER: &str = include_str!("prompts/changes_header.txt");
const CHANGES_FOOTER: &str = include_str!("prompts/changes_footer.txt");

/// Build the session prompt from built-in + school + project layers.
pub fn build_session_prompt(
    school_name: &str,
    description: Option<&str>,
    school_session_prompt: &str,
    project_session_prompt: &str,
    skills_dir: &Path,
    changes: &[SkillChange],
) -> String {
    let mut parts = Vec::new();

    let mut builtin = format!("School: {school_name}");
    if let Some(desc) = description {
        builtin.push_str(&format!("\n\n{desc}"));
    }
    builtin.push_str(&format!("\n\n{SCHOOL_EDITABLE}"));
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

    let previous_skills = skills_dir.join("skills").join("previous-skills");
    if previous_skills.exists() {
        parts.push(PREVIOUS_SKILLS.to_string());
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

    let mut lines = vec![CHANGES_HEADER.to_string()];
    for (label, names) in [("Added", added), ("Updated", modified), ("Removed", removed)] {
        for name in names {
            lines.push(format!("- {label}: `{name}`"));
        }
    }
    lines.push(format!("\n{CHANGES_FOOTER}"));

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
    fn builtin_only() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", None, "", "", &dir, &[]);
        assert!(prompt.contains("School: Acme"));
        assert!(prompt.contains("ace school propose"));
    }

    #[test]
    fn with_description() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", Some("Acme engineering"), "", "", &dir, &[]);
        assert!(prompt.contains("School: Acme"));
        assert!(prompt.contains("Acme engineering"));
    }

    #[test]
    fn school_and_project_prompts() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", None, "Use Rust.", "PostgreSQL project.", &dir, &[]);
        assert!(prompt.contains("Use Rust."));
        assert!(prompt.contains("PostgreSQL project."));
        let school_pos = prompt.find("Use Rust.").expect("school prompt present");
        let project_pos = prompt.find("PostgreSQL project.").expect("project prompt present");
        assert!(school_pos < project_pos, "school before project");
    }

    #[test]
    fn skips_empty_layers() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", None, "", "Only project.", &dir, &[]);
        assert!(!prompt.contains("\n\n\n"), "no triple newlines from skipped school prompt");
        assert!(prompt.contains("Only project."));
    }

    #[test]
    fn injects_previous_skills_guidance() {
        let fix = TempDir::new("ace-test-prompt-previous");
        let prev = fix.path().join("skills").join("previous-skills");
        std::fs::create_dir_all(&prev).expect("create previous-skills dir");

        let prompt = build_session_prompt("Acme", None, "", "", fix.path(), &[]);
        assert!(prompt.contains("unconsolidated skills"));
        assert!(prompt.contains("previous-skills"));
    }

    #[test]
    fn no_previous_skills_no_injection() {
        let fix = TempDir::new("ace-test-prompt-no-previous");
        std::fs::create_dir_all(fix.path().join("skills")).expect("create skills dir");

        let prompt = build_session_prompt("Acme", None, "", "", fix.path(), &[]);
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

        let prompt = build_session_prompt("Acme", None, "", "", &dir, &changes);
        assert!(prompt.contains("School skills were updated"));
        assert!(prompt.contains("- Added: `new-skill`"));
        assert!(prompt.contains("- Updated: `existing`"));
        assert!(prompt.contains("- Removed: `old-skill`"));
    }

    #[test]
    fn no_changes_no_summary() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", None, "", "", &dir, &[]);
        assert!(!prompt.contains("updated since your last session"));
    }
}
