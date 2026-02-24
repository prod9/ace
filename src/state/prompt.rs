use std::path::Path;

/// Build the session prompt from built-in + school + project layers.
pub fn build_session_prompt(
    school_name: &str,
    description: Option<&str>,
    school_session_prompt: &str,
    project_session_prompt: &str,
    skills_dir: &Path,
) -> String {
    let mut parts = Vec::new();

    // Built-in section
    let mut builtin = format!("School: {school_name}");
    if let Some(desc) = description {
        builtin.push_str(&format!("\n\n{desc}"));
    }
    builtin.push_str(
        "\n\nSkills are loaded from the linked school and are editable. \
         If you modify any skill files during this session, \
         run `ace school propose` afterward to propose changes back to the school repo.",
    );
    parts.push(builtin);

    if !school_session_prompt.is_empty() {
        parts.push(school_session_prompt.to_string());
    }

    if !project_session_prompt.is_empty() {
        parts.push(project_session_prompt.to_string());
    }

    let previous_skills = skills_dir.join("skills").join("previous-skills");
    if previous_skills.exists() {
        parts.push(
            "This project has unconsolidated skills in `.claude/skills/previous-skills/`. \
             Before starting work:\n\
             1. Review previous skills and current school skills (symlinked in `.claude/skills/`)\n\
             2. For each previous skill: merge into an existing school skill (edit through symlink), \
                or add as a new school skill (create a new dir in `.claude/skills/`)\n\
             3. After consolidation, run `ace school propose` to submit changes upstream\n\
             4. Delete `.claude/skills/previous-skills/` when done"
                .to_string(),
        );
    }

    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nonexistent_dir() -> std::path::PathBuf {
        std::path::PathBuf::from("/tmp/ace-test-prompt-nonexistent")
    }

    #[test]
    fn builtin_only() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", None, "", "", &dir);
        assert!(prompt.contains("School: Acme"));
        assert!(prompt.contains("ace school propose"));
    }

    #[test]
    fn with_description() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", Some("Acme engineering"), "", "", &dir);
        assert!(prompt.contains("School: Acme"));
        assert!(prompt.contains("Acme engineering"));
    }

    #[test]
    fn school_and_project_prompts() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", None, "Use Rust.", "PostgreSQL project.", &dir);
        assert!(prompt.contains("Use Rust."));
        assert!(prompt.contains("PostgreSQL project."));
        let school_pos = prompt.find("Use Rust.").expect("school prompt present");
        let project_pos = prompt.find("PostgreSQL project.").expect("project prompt present");
        assert!(school_pos < project_pos, "school before project");
    }

    #[test]
    fn skips_empty_layers() {
        let dir = nonexistent_dir();
        let prompt = build_session_prompt("Acme", None, "", "Only project.", &dir);
        assert!(!prompt.contains("\n\n\n"), "no triple newlines from skipped school prompt");
        assert!(prompt.contains("Only project."));
    }

    #[test]
    fn injects_previous_skills_guidance() {
        let dir = std::env::temp_dir().join("ace-test-prompt-previous");
        let _ = std::fs::remove_dir_all(&dir);
        let prev = dir.join("skills").join("previous-skills");
        std::fs::create_dir_all(&prev).expect("create previous-skills dir");

        let prompt = build_session_prompt("Acme", None, "", "", &dir);
        assert!(prompt.contains("unconsolidated skills"));
        assert!(prompt.contains("previous-skills"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_previous_skills_no_injection() {
        let dir = std::env::temp_dir().join("ace-test-prompt-no-previous");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("skills")).expect("create skills dir");

        let prompt = build_session_prompt("Acme", None, "", "", &dir);
        assert!(!prompt.contains("unconsolidated"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
