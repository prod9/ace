use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use super::prepare_school::PrepareError;
use crate::ace::Ace;
use crate::config;

impl UpdateOutcome {
    pub fn emit(&self, ace: &mut Ace) {
        match self {
            UpdateOutcome::Embedded | UpdateOutcome::Fresh => {}
            UpdateOutcome::SwitchedBranch { from } => {
                ace.hint(&format!(
                    "Switched school cache from branch {from} back to main"
                ));
            }
            UpdateOutcome::Updated { changes } if changes.is_empty() => {
                ace.done("School updated (no skill changes)");
            }
            UpdateOutcome::Updated { changes } => {
                let summary: Vec<String> = changes.iter().map(|c| {
                    let prefix = match c.kind {
                        ChangeKind::Added => "+",
                        ChangeKind::Modified => "~",
                        ChangeKind::Removed => "-",
                    };
                    format!("{prefix}{}", c.name)
                }).collect();
                ace.done(&format!("School updated ({})", summary.join(", ")));
            }
            UpdateOutcome::Dirty { on_main: true, .. } => {
                ace.warn("school has local changes — updates blocked");
                ace.hint("Skills may be outdated until changes are proposed.");
                ace.hint("Ask your AI assistant to propose the changes — it knows how.");
            }
            UpdateOutcome::Dirty { on_main: false, branch } => {
                ace.warn(&format!(
                    "school is on branch {branch} with uncommitted changes — updates blocked"
                ));
                ace.hint("Skills may be outdated. Ask your AI assistant to propose the changes — it knows how.");
            }
            UpdateOutcome::AheadOfOrigin { cache_path } => {
                ace.warn(&format!("school has local commits at {cache_path}"));
                ace.hint("Propose changes back to the school repo, or resolve manually.");
            }
            UpdateOutcome::Diverged { error } => {
                ace.warn(&format!("school has diverged from origin/main: {error}"));
                ace.hint("Propose changes back to the school repo, or resolve manually.");
            }
        }
    }
}

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

/// Outcome of a school cache update — carries data for the caller to act on.
#[derive(Debug)]
pub enum UpdateOutcome {
    /// Embedded school, no cache to update.
    Embedded,
    /// Cache is fresh (within cooldown), no fetch needed.
    Fresh,
    /// Was on a non-main branch (clean), switched back to main.
    SwitchedBranch { from: String },
    /// Fetched and fast-forwarded successfully.
    Updated { changes: Vec<SkillChange> },
    /// Working tree has uncommitted changes.
    Dirty { on_main: bool, branch: String },
    /// Local commits ahead of origin (can't fast-forward).
    AheadOfOrigin { cache_path: String },
    /// Local and remote have diverged (ff-only merge failed).
    Diverged { error: String },
}

/// Git fetch + ff-only merge school cache to latest origin/main.
/// Dirty, ahead, or diverged caches are warned but not errors — update is skipped.
pub struct Update<'a> {
    pub specifier: &'a str,
    pub project_dir: &'a Path,
    pub force: bool,
}

impl Update<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<UpdateOutcome, PrepareError> {
        // -- resolve school paths --

        let school_paths = config::school_paths::resolve(self.project_dir, self.specifier)?;

        let Some(cache) = &school_paths.cache else {
            return Ok(UpdateOutcome::Embedded);
        };

        if !cache.join(".git").exists() {
            return Err(PrepareError::Clone(format!(
                "school not installed: {}",
                self.specifier
            )));
        }

        // -- ensure working tree is clean and on main --

        let git = ace.git(cache);
        let branch = git
            .current_branch()
            .map_err(|e| PrepareError::Clone(e.to_string()))?;
        let on_main = branch == "main";
        let dirty = git
            .is_dirty()
            .map_err(|e| PrepareError::Clone(e.to_string()))?;

        if dirty {
            return Ok(UpdateOutcome::Dirty {
                on_main,
                branch: branch.to_string(),
            });
        }

        let switched_from = if !on_main {
            let from = branch.clone();
            git.checkout_branch("main")
                .map_err(|e| PrepareError::Clone(e.to_string()))?;
            Some(from)
        } else {
            None
        };

        if !self.force && !is_stale(cache) {
            return if let Some(from) = switched_from {
                Ok(UpdateOutcome::SwitchedBranch { from })
            } else {
                Ok(UpdateOutcome::Fresh)
            };
        }

        // -- fetch and fast-forward --

        let old_head = git
            .rev_parse("HEAD")
            .map_err(|e| PrepareError::Clone(e.to_string()))?;

        ace.progress(&format!("Fetching {}", self.specifier));
        git.fetch("origin", "main")
            .map_err(|e| PrepareError::Clone(e.to_string()))?;

        if git
            .is_ahead_of("origin/main")
            .map_err(|e| PrepareError::Clone(e.to_string()))?
        {
            return Ok(UpdateOutcome::AheadOfOrigin {
                cache_path: cache.display().to_string(),
            });
        }

        if let Err(e) = git.merge_ff_only("origin/main") {
            return Ok(UpdateOutcome::Diverged {
                error: e.to_string(),
            });
        }

        // -- collect skill changes --

        let new_head = git
            .rev_parse("HEAD")
            .map_err(|e| PrepareError::Clone(e.to_string()))?;

        let changes = diff_skill_changes(&git, &old_head, &new_head);

        Ok(UpdateOutcome::Updated { changes })
    }
}

fn diff_skill_changes(
    git: &crate::git::Git<'_>,
    old: &str,
    new: &str,
) -> Vec<SkillChange> {
    if old == new {
        return Vec::new();
    }

    match git.diff_name_status(old, new, Some("skills/")) {
        Ok(stdout) => parse_diff_output(&stdout),
        Err(_) => Vec::new(),
    }
}

/// Check if the last fetch was longer ago than FETCH_COOLDOWN.
/// Returns true (stale) if FETCH_HEAD is missing or old.
fn is_stale(repo: &Path) -> bool {
    let fetch_head = repo.join(".git/FETCH_HEAD");
    let age = fetch_head
        .metadata()
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
