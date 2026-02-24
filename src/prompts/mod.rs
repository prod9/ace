pub const SESSION: &str = include_str!("session.md");
pub const PREVIOUS_SKILLS: &str = include_str!("previous_skills.md");
pub const CHANGES_HEADER: &str = include_str!("changes_header.md");
pub const CHANGES_FOOTER: &str = include_str!("changes_footer.md");

use std::path::Path;

const UNKNOWN_SKILLS_DIR: &str =
    "<UNKNOWN_SKILLS_DIR: flag this to the user and do not modify skills>";

/// Runtime values for prompt template placeholders.
pub struct PromptCtx {
    skills_dir: String,
}

impl PromptCtx {
    pub fn from_skills_dir(path: &Path) -> Self {
        let skills_dir = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(UNKNOWN_SKILLS_DIR)
            .to_string();

        Self { skills_dir }
    }
}

/// Replace `{key}` placeholders in a template string with values from `ctx`.
pub fn render(template: &str, ctx: &PromptCtx) -> String {
    template.replace("{skills_dir}", &ctx.skills_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_replaces_skills_dir() {
        let ctx = PromptCtx::from_skills_dir(Path::new("/project/.opencode"));
        let result = render("look in {skills_dir}/skills/ for files", &ctx);
        assert_eq!(result, "look in .opencode/skills/ for files");
    }

    #[test]
    fn render_leaves_unknown_placeholders() {
        let ctx = PromptCtx::from_skills_dir(Path::new("/project/.claude"));
        let result = render("{unknown} stays {skills_dir} replaced", &ctx);
        assert_eq!(result, "{unknown} stays .claude replaced");
    }

    #[test]
    fn render_flags_unknown_skills_dir() {
        let ctx = PromptCtx::from_skills_dir(Path::new("/"));
        let result = render("{skills_dir}/skills/", &ctx);
        assert!(result.contains("UNKNOWN_SKILLS_DIR"));
        assert!(result.contains("do not modify"));
    }
}
