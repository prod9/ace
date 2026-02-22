/// Build the session prompt from built-in + school + project layers.
pub fn build_session_prompt(
    school_name: &str,
    description: Option<&str>,
    school_session_prompt: &str,
    project_session_prompt: &str,
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

    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_only() {
        let prompt = build_session_prompt("Acme", None, "", "");
        assert!(prompt.contains("School: Acme"));
        assert!(prompt.contains("ace school propose"));
    }

    #[test]
    fn with_description() {
        let prompt = build_session_prompt("Acme", Some("Acme engineering"), "", "");
        assert!(prompt.contains("School: Acme"));
        assert!(prompt.contains("Acme engineering"));
    }

    #[test]
    fn school_and_project_prompts() {
        let prompt = build_session_prompt("Acme", None, "Use Rust.", "PostgreSQL project.");
        assert!(prompt.contains("Use Rust."));
        assert!(prompt.contains("PostgreSQL project."));
        let school_pos = prompt.find("Use Rust.").expect("school prompt present");
        let project_pos = prompt.find("PostgreSQL project.").expect("project prompt present");
        assert!(school_pos < project_pos, "school before project");
    }

    #[test]
    fn skips_empty_layers() {
        let prompt = build_session_prompt("Acme", None, "", "Only project.");
        assert!(!prompt.contains("\n\n\n"), "no triple newlines from skipped school prompt");
        assert!(prompt.contains("Only project."));
    }
}
