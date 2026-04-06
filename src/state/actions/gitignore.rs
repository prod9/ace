use std::path::Path;

use crate::ace::Ace;
use crate::templates;
use super::link::SCHOOL_FOLDERS;

const MARKER_START: &str = "# ACE-managed — do not edit this block.";
const MARKER_END: &str = "# end ACE";

pub struct UpdateGitignore<'a> {
    pub project_dir: &'a Path,
    pub backend_dir: &'a str,
}

impl UpdateGitignore<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<(), std::io::Error> {
        let path = self.project_dir.join(".gitignore");
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let block = build_block(self.backend_dir);

        let new_content = if existing.contains(MARKER_START) {
            replace_block(&existing, &block)
        } else {
            append_block(&existing, &block)
        };

        if new_content == existing {
            return Ok(());
        }

        std::fs::write(&path, &new_content)?;
        ace.done("Updated .gitignore with ACE patterns");
        Ok(())
    }
}

fn build_block(backend_dir: &str) -> String {
    let folders = SCHOOL_FOLDERS.iter()
        .map(|f| format!("{backend_dir}/{f}"))
        .collect::<Vec<_>>()
        .join("\n");

    let tpl = templates::Template::parse(templates::builtins::PROJECT_GITIGNORE);
    tpl.substitute(&std::collections::HashMap::from([
        ("folders".to_string(), folders),
    ]))
}

fn replace_block(content: &str, block: &str) -> String {
    let mut result = String::new();
    let mut in_block = false;
    let mut replaced = false;

    for line in content.lines() {
        if line.trim() == MARKER_START {
            in_block = true;
            if !replaced {
                result.push_str(block);
                replaced = true;
            }
            continue;
        }

        if in_block {
            if line.trim() == MARKER_END {
                in_block = false;
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

fn append_block(content: &str, block: &str) -> String {
    let mut result = content.to_string();

    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    if !result.is_empty() && !result.ends_with("\n\n") {
        result.push('\n');
    }

    result.push_str(block);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_block_uses_backend_dir() {
        let block = build_block(".claude");
        assert!(block.contains(".claude/skills"));
        assert!(block.contains(".claude/rules"));
        assert!(block.contains(".claude/commands"));
        assert!(block.contains(".claude/agents"));
        assert!(block.contains("ace.local.toml"));
        assert!(block.contains(MARKER_START));
        assert!(block.contains(MARKER_END));
    }

    #[test]
    fn build_block_agents() {
        let block = build_block(".agents");
        assert!(block.contains(".agents/skills"));
        assert!(block.contains(".agents/agents"));
        assert!(block.contains("ace.local.toml"));
    }

    #[test]
    fn append_to_empty() {
        let block = build_block(".claude");
        let result = append_block("", &block);
        assert_eq!(result, block);
    }

    #[test]
    fn append_to_existing() {
        let block = build_block(".claude");
        let result = append_block("node_modules/\n", &block);
        assert!(result.starts_with("node_modules/\n"));
        assert!(result.contains(MARKER_START));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn append_adds_blank_line_separator() {
        let block = build_block(".claude");
        let result = append_block("node_modules/\n", &block);
        assert!(result.contains("node_modules/\n\n#"));
    }

    #[test]
    fn replace_existing_block() {
        let original = format!(
            "node_modules/\n{MARKER_START}\n.old/skills/\n{MARKER_END}\n.env\n"
        );
        let block = build_block(".claude");
        let result = replace_block(&original, &block);

        assert!(result.contains("node_modules/"));
        assert!(result.contains(".env"));
        assert!(result.contains(".claude/skills"));
        assert!(!result.contains(".old/skills/"));
    }

    #[test]
    fn replace_preserves_surrounding_content() {
        let original = format!(
            "before\n{MARKER_START}\nold stuff\n{MARKER_END}\nafter\n"
        );
        let block = build_block(".claude");
        let result = replace_block(&original, &block);

        assert!(result.contains("before\n"));
        assert!(result.contains("after\n"));
    }

    #[test]
    fn idempotent_when_unchanged() {
        let block = build_block(".claude");
        let content = append_block("", &block);
        let replaced = replace_block(&content, &block);

        // Both should contain the same block
        assert!(replaced.contains(MARKER_START));
        assert!(replaced.contains(".claude/skills"));
    }
}
