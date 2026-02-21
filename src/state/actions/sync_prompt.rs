use std::path::Path;

use crate::session::Session;

#[derive(Debug, thiserror::Error)]
pub enum SyncPromptError {
    #[error("failed to read skills dir: {0}")]
    ReadSkills(std::io::Error),
}

pub struct SyncPrompt<'a> {
    pub school_root: &'a Path,
    pub school_name: &'a str,
    pub school_description: Option<&'a str>,
}

impl SyncPrompt<'_> {
    pub fn run(&self, _session: &mut Session<'_>) -> Result<String, SyncPromptError> {
        let skills = list_skills(self.school_root)?;
        let prompt = build_prompt(self.school_name, self.school_description, &skills);
        Ok(prompt)
    }
}

fn list_skills(school_root: &Path) -> Result<Vec<String>, SyncPromptError> {
    let skills_dir = school_root.join("skills");
    if !skills_dir.is_dir() {
        return Ok(vec![]);
    }

    let mut names = Vec::new();
    let entries = std::fs::read_dir(&skills_dir).map_err(SyncPromptError::ReadSkills)?;
    for entry in entries {
        let entry = entry.map_err(SyncPromptError::ReadSkills)?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

fn build_prompt(school_name: &str, description: Option<&str>, skills: &[String]) -> String {
    let mut parts = Vec::new();

    parts.push(format!("School: {school_name}"));
    if let Some(desc) = description {
        parts.push(desc.to_string());
    }

    if !skills.is_empty() {
        let list = skills.join(", ");
        parts.push(format!("Available skills: {list}"));
    }

    parts.push(
        "Skills are loaded from the linked school and are editable. \
         If you modify any skill files during this session, \
         run `ace school propose` afterward to propose changes back to the school repo."
            .to_string(),
    );

    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_prompt_with_skills() {
        let prompt = build_prompt("Acme", Some("Acme engineering"), &["rust".into(), "testing".into()]);
        assert!(prompt.contains("School: Acme"), "should contain school name");
        assert!(prompt.contains("Acme engineering"), "should contain description");
        assert!(prompt.contains("rust, testing"), "should list skills");
        assert!(prompt.contains("ace school propose"), "should mention propose workflow");
    }

    #[test]
    fn build_prompt_no_skills() {
        let prompt = build_prompt("Acme", None, &[]);
        assert!(prompt.contains("School: Acme"), "should contain school name");
        assert!(!prompt.contains("Available skills"), "should not list skills section");
        assert!(prompt.contains("ace school propose"), "should mention propose workflow");
    }

    #[test]
    fn list_skills_missing_dir() {
        let dir = Path::new("/tmp/ace_test_nonexistent_school_root");
        let skills = list_skills(dir).expect("missing dir should return empty vec");
        assert!(skills.is_empty());
    }
}
