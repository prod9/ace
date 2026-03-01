use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use crate::ace::Ace;
use crate::config;
use super::prepare::PrepareError;

const FETCH_COOLDOWN: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug)]
pub struct SkillChange {
    pub name: String,
    pub kind: ChangeKind,
}

#[derive(Debug, Default)]
pub struct UpdateResult {
    pub changes: Vec<SkillChange>,
}

/// Git fetch + reset school cache to latest origin/main.
/// Returns `PrepareError::DirtyCache` if cache has uncommitted changes.
pub struct Update<'a> {
    pub specifier: &'a str,
    pub project_dir: &'a Path,
}

impl Update<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<UpdateResult, PrepareError> {
        let school_paths = config::school_paths::resolve(self.project_dir, self.specifier)?;

        let cache = match &school_paths.cache {
            Some(c) => c,
            None => return Ok(UpdateResult::default()), // embedded school
        };

        if !cache.join(".git").exists() {
            return Err(PrepareError::Clone(format!(
                "school not installed: {}", self.specifier
            )));
        }

        let git = ace.git(cache);

        if git.is_dirty().map_err(|e| PrepareError::Clone(e.to_string()))? {
            return Err(PrepareError::DirtyCache);
        }

        if !is_stale(cache) {
            return Ok(UpdateResult::default());
        }

        ace.progress(&format!("Fetching {}", self.specifier));
        let git = ace.git(cache);
        git.fetch_shallow("origin", "main")
            .map_err(|e| PrepareError::Clone(e.to_string()))?;
        ace.done(&format!("Fetched {}", self.specifier));

        let changes = {
            let git = ace.git(cache);
            match git.diff_name_status("HEAD", "origin/main", Some("skills/")) {
                Ok(stdout) => parse_diff_output(&stdout),
                Err(_) => Vec::new(),
            }
        };

        ace.git(cache).reset_hard("origin/main")
            .map_err(|e| PrepareError::Clone(e.to_string()))?;

        Ok(UpdateResult { changes })
    }
}

/// Check if the last fetch was longer ago than FETCH_COOLDOWN.
/// Returns true (stale) if FETCH_HEAD is missing or old.
fn is_stale(repo: &Path) -> bool {
    let fetch_head = repo.join(".git/FETCH_HEAD");
    let age = fetch_head.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok());

    match age {
        Some(d) => d > FETCH_COOLDOWN,
        None => true,
    }
}

fn parse_diff_output(output: &str) -> Vec<SkillChange> {
    let mut seen = HashSet::new();
    let mut changes = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (status, path) = match line.split_once('\t') {
            Some(pair) => pair,
            None => continue,
        };

        // Extract skill name from "skills/{name}/..."
        let rest = match path.strip_prefix("skills/") {
            Some(r) => r,
            None => continue,
        };
        let name = rest.split('/').next().unwrap_or(rest);
        if name.is_empty() {
            continue;
        }

        if !seen.insert(name.to_string()) {
            continue;
        }

        let kind = match status.chars().next() {
            Some('A') => ChangeKind::Added,
            Some('D') => ChangeKind::Removed,
            _ => ChangeKind::Modified,
        };

        changes.push(SkillChange {
            name: name.to_string(),
            kind,
        });
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_added_modified_removed() {
        let output = "A\tskills/new-skill/SKILL.md\n\
                       M\tskills/existing/SKILL.md\n\
                       D\tskills/old-skill/SKILL.md\n";
        let changes = parse_diff_output(output);
        assert_eq!(changes.len(), 3);
        assert_eq!(changes[0].name, "new-skill");
        assert_eq!(changes[0].kind, ChangeKind::Added);
        assert_eq!(changes[1].name, "existing");
        assert_eq!(changes[1].kind, ChangeKind::Modified);
        assert_eq!(changes[2].name, "old-skill");
        assert_eq!(changes[2].kind, ChangeKind::Removed);
    }

    #[test]
    fn dedup_by_skill_name() {
        let output = "M\tskills/my-skill/SKILL.md\n\
                       M\tskills/my-skill/prompt.md\n\
                       A\tskills/other/SKILL.md\n";
        let changes = parse_diff_output(output);
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].name, "my-skill");
        assert_eq!(changes[1].name, "other");
    }

    #[test]
    fn ignores_non_skills_paths() {
        let output = "M\tREADME.md\n\
                       M\tschool.toml\n\
                       A\tskills/real/SKILL.md\n";
        let changes = parse_diff_output(output);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].name, "real");
    }

    #[test]
    fn empty_output() {
        assert!(parse_diff_output("").is_empty());
        assert!(parse_diff_output("  \n  \n").is_empty());
    }

    #[test]
    fn rename_treated_as_modified() {
        let output = "R100\tskills/old-name/SKILL.md\tskills/new-name/SKILL.md\n";
        let changes = parse_diff_output(output);
        // R lines have the tab-separated old path first; parse picks up old-name as Modified
        assert!(!changes.is_empty());
    }
}
