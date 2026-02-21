use std::path::Path;

use crate::session::Session;
use super::setup::SetupError;

/// Symlinks skills from the school's `skills/` directory into the project's
/// `.ace/skills/` directory. Uses symlinks so all projects sharing a school
/// see the same skill versions from a single local clone.
pub struct SyncSkills<'a> {
    pub school_root: &'a Path,
    pub project_dir: &'a Path,
}

impl SyncSkills<'_> {
    pub fn run(&self, _session: &mut Session<'_>) -> Result<SyncResult, SetupError> {
        let school_skills = self.school_root.join("skills");
        if !school_skills.exists() {
            return Ok(SyncResult { synced: 0 });
        }

        let project_skills = self.project_dir.join(".ace").join("skills");
        std::fs::create_dir_all(&project_skills)
            .map_err(|e| SetupError::WriteConfig(e))?;

        let mut synced = 0;
        let entries = std::fs::read_dir(&school_skills)
            .map_err(|e| SetupError::WriteConfig(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| SetupError::WriteConfig(e))?;
            let file_type = entry.file_type().map_err(|e| SetupError::WriteConfig(e))?;
            if !file_type.is_dir() {
                continue;
            }

            let skill_name = entry.file_name();
            let link_path = project_skills.join(&skill_name);
            let target = entry.path();

            // Remove existing symlink/dir before creating new one
            if link_path.exists() || link_path.symlink_metadata().is_ok() {
                if link_path.symlink_metadata()
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false)
                {
                    std::fs::remove_file(&link_path)
                        .map_err(|e| SetupError::WriteConfig(e))?;
                } else {
                    // Not a symlink — don't clobber user files
                    continue;
                }
            }

            std::os::unix::fs::symlink(&target, &link_path)
                .map_err(|e| SetupError::WriteConfig(e))?;
            synced += 1;
        }

        Ok(SyncResult { synced })
    }
}

pub struct SyncResult {
    pub synced: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;

    fn make_session(state: &mut State) -> Session<'_> {
        Session { state }
    }

    #[test]
    fn sync_no_skills_dir() {
        let dir = std::env::temp_dir().join("ace-test-sync-no-skills");
        let _ = std::fs::remove_dir_all(&dir);
        let school = dir.join("school");
        let project = dir.join("project");
        std::fs::create_dir_all(&school).expect("create school dir");
        std::fs::create_dir_all(&project).expect("create project dir");

        let mut state = State::empty();
        let mut session = make_session(&mut state);
        let result = SyncSkills {
            school_root: &school,
            project_dir: &project,
        }
        .run(&mut session)
        .expect("sync should succeed with no skills dir");

        assert_eq!(result.synced, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sync_creates_symlinks() {
        let dir = std::env::temp_dir().join("ace-test-sync-symlinks");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        // Create two skill directories with content
        std::fs::create_dir_all(skills.join("git-commit")).expect("create skill dir");
        std::fs::write(
            skills.join("git-commit").join("SKILL.md"),
            "# Git Commit",
        )
        .expect("write skill");

        std::fs::create_dir_all(skills.join("code-review")).expect("create skill dir");
        std::fs::write(
            skills.join("code-review").join("SKILL.md"),
            "# Code Review",
        )
        .expect("write skill");

        std::fs::create_dir_all(&project).expect("create project dir");

        let mut state = State::empty();
        let mut session = make_session(&mut state);
        let result = SyncSkills {
            school_root: &school,
            project_dir: &project,
        }
        .run(&mut session)
        .expect("sync should create symlinks");

        assert_eq!(result.synced, 2);

        let link = project.join(".ace").join("skills").join("git-commit");
        assert!(link.symlink_metadata().expect("link should exist").file_type().is_symlink());

        // Verify content is accessible through symlink
        let content = std::fs::read_to_string(link.join("SKILL.md")).expect("read through symlink");
        assert_eq!(content, "# Git Commit");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sync_replaces_stale_symlinks() {
        let dir = std::env::temp_dir().join("ace-test-sync-replace");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        std::fs::create_dir_all(skills.join("my-skill")).expect("create skill dir");
        std::fs::create_dir_all(&project).expect("create project dir");

        // Create a stale symlink
        let project_skills = project.join(".ace").join("skills");
        std::fs::create_dir_all(&project_skills).expect("mkdir");
        let stale_target = dir.join("nonexistent");
        std::os::unix::fs::symlink(&stale_target, project_skills.join("my-skill"))
            .expect("create stale symlink");

        let mut state = State::empty();
        let mut session = make_session(&mut state);
        let result = SyncSkills {
            school_root: &school,
            project_dir: &project,
        }
        .run(&mut session)
        .expect("sync should replace stale symlink");

        assert_eq!(result.synced, 1);
        let link = project_skills.join("my-skill");
        let target = std::fs::read_link(&link).expect("read symlink target");
        assert_eq!(target, skills.join("my-skill"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sync_skips_non_symlink_dirs() {
        let dir = std::env::temp_dir().join("ace-test-sync-skip");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        std::fs::create_dir_all(skills.join("my-skill")).expect("create skill dir");

        // Create a real directory (not symlink) in the project skills dir
        let project_skills = project.join(".ace").join("skills");
        std::fs::create_dir_all(project_skills.join("my-skill")).expect("create real dir");
        std::fs::write(
            project_skills.join("my-skill").join("local.md"),
            "local override",
        )
        .expect("write local file");

        let mut state = State::empty();
        let mut session = make_session(&mut state);
        let result = SyncSkills {
            school_root: &school,
            project_dir: &project,
        }
        .run(&mut session)
        .expect("sync should skip real dirs");

        // Should skip because it's a real dir, not clobber
        assert_eq!(result.synced, 0);
        let content = std::fs::read_to_string(
            project_skills.join("my-skill").join("local.md"),
        )
        .expect("local file should still exist");
        assert_eq!(content, "local override");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
