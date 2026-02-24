use std::path::Path;

use crate::config;
use crate::session::Session;
use super::setup::SetupError;

const PREVIOUS_SKILLS_DIR: &str = "previous-skills";

/// Symlink school skills from cache into the project.
pub struct Link<'a> {
    pub specifier: &'a str,
    pub project_dir: &'a Path,
    pub skills_dir: &'a str,
}

impl Link<'_> {
    pub fn run(&self, _session: &mut Session<'_>) -> Result<LinkResult, SetupError> {
        let school_paths = config::school_paths::resolve(self.project_dir, self.specifier)?;
        let school_skills = school_paths.root.join("skills");
        if !school_skills.exists() {
            return Ok(LinkResult::default());
        }

        let project_skills = self.project_dir.join(self.skills_dir).join("skills");
        std::fs::create_dir_all(&project_skills)
            .map_err(SetupError::WriteConfig)?;

        let moved = adopt_previous_skills(&project_skills)?;

        let mut linked = 0;
        let entries = std::fs::read_dir(&school_skills)
            .map_err(SetupError::WriteConfig)?;

        for entry in entries {
            let entry = entry.map_err(SetupError::WriteConfig)?;
            let file_type = entry.file_type().map_err(SetupError::WriteConfig)?;
            if !file_type.is_dir() {
                continue;
            }

            let skill_name = entry.file_name();
            let link_path = project_skills.join(&skill_name);
            let target = entry.path();

            match link_status(&link_path) {
                LinkStatus::Absent => {}
                LinkStatus::RealDir => continue,
                LinkStatus::Symlink(current) if current == target => continue,
                LinkStatus::Symlink(_) => {
                    std::fs::remove_file(&link_path)
                        .map_err(SetupError::WriteConfig)?;
                }
            }

            std::os::unix::fs::symlink(&target, &link_path)
                .map_err(SetupError::WriteConfig)?;
            linked += 1;
        }

        Ok(LinkResult { linked, moved })
    }
}

#[derive(Debug, Default)]
pub struct LinkResult {
    pub linked: usize,
    pub moved: Vec<String>,
}

/// On first setup (no symlinks yet), move real skill dirs into `previous-skills/`.
fn adopt_previous_skills(project_skills: &Path) -> Result<Vec<String>, SetupError> {
    let entries = match std::fs::read_dir(project_skills) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };

    let mut real_dirs = Vec::new();
    let mut has_symlinks = false;

    for entry in entries {
        let entry = entry.map_err(SetupError::WriteConfig)?;
        let name = entry.file_name();
        if name == PREVIOUS_SKILLS_DIR {
            continue;
        }

        match link_status(&entry.path()) {
            LinkStatus::RealDir => real_dirs.push(name),
            LinkStatus::Symlink(_) => has_symlinks = true,
            LinkStatus::Absent => {}
        }
    }

    if real_dirs.is_empty() || has_symlinks {
        return Ok(Vec::new());
    }

    let prev_dir = project_skills.join(PREVIOUS_SKILLS_DIR);
    if prev_dir.exists() {
        return Err(SetupError::WriteConfig(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists — consolidate or remove it first",
                prev_dir.display()
            ),
        )));
    }

    std::fs::create_dir_all(&prev_dir).map_err(SetupError::WriteConfig)?;

    let mut moved = Vec::new();
    for name in real_dirs {
        let src = project_skills.join(&name);
        let dst = prev_dir.join(&name);
        std::fs::rename(&src, &dst).map_err(SetupError::WriteConfig)?;
        moved.push(name.to_string_lossy().into_owned());
    }

    Ok(moved)
}

enum LinkStatus {
    Absent,
    Symlink(std::path::PathBuf),
    RealDir,
}

fn link_status(path: &Path) -> LinkStatus {
    let is_symlink = path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);

    if is_symlink {
        let target = std::fs::read_link(path).unwrap_or_default();
        LinkStatus::Symlink(target)
    } else if path.exists() {
        LinkStatus::RealDir
    } else {
        LinkStatus::Absent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_no_skills_dir() {
        let dir = std::env::temp_dir().join("ace-test-link-no-skills");
        let _ = std::fs::remove_dir_all(&dir);
        let school = dir.join("school");
        let project = dir.join("project");
        std::fs::create_dir_all(&school).expect("create school dir");
        std::fs::create_dir_all(&project).expect("create project dir");

        let result = link_skills(&school, &project).expect("should succeed");
        assert_eq!(result.linked, 0);
        assert!(result.moved.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn link_creates_symlinks() {
        let dir = std::env::temp_dir().join("ace-test-link-symlinks");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        std::fs::create_dir_all(skills.join("git-commit")).expect("create skill dir");
        std::fs::write(skills.join("git-commit").join("SKILL.md"), "# Git Commit")
            .expect("write skill");

        std::fs::create_dir_all(skills.join("code-review")).expect("create skill dir");
        std::fs::write(skills.join("code-review").join("SKILL.md"), "# Code Review")
            .expect("write skill");

        std::fs::create_dir_all(&project).expect("create project dir");

        let result = link_skills(&school, &project).expect("should create symlinks");
        assert_eq!(result.linked, 2);

        let link = project.join(".claude").join("skills").join("git-commit");
        assert!(link.symlink_metadata().expect("link should exist").file_type().is_symlink());

        let content = std::fs::read_to_string(link.join("SKILL.md")).expect("read through symlink");
        assert_eq!(content, "# Git Commit");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn link_skips_matching_symlinks() {
        let dir = std::env::temp_dir().join("ace-test-link-skip-matching");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        std::fs::create_dir_all(skills.join("my-skill")).expect("create skill dir");

        let project_skills = project.join(".claude").join("skills");
        std::fs::create_dir_all(&project_skills).expect("mkdir");
        std::os::unix::fs::symlink(skills.join("my-skill"), project_skills.join("my-skill"))
            .expect("create symlink");

        let result = link_skills(&school, &project).expect("should skip existing");
        assert_eq!(result.linked, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn link_replaces_stale_symlinks() {
        let dir = std::env::temp_dir().join("ace-test-link-replace");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        std::fs::create_dir_all(skills.join("my-skill")).expect("create skill dir");
        std::fs::create_dir_all(&project).expect("create project dir");

        let project_skills = project.join(".claude").join("skills");
        std::fs::create_dir_all(&project_skills).expect("mkdir");
        std::os::unix::fs::symlink(dir.join("nonexistent"), project_skills.join("my-skill"))
            .expect("create stale symlink");

        let result = link_skills(&school, &project).expect("should replace stale");
        assert_eq!(result.linked, 1);

        let target = std::fs::read_link(project_skills.join("my-skill")).expect("read link");
        assert_eq!(target, skills.join("my-skill"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn link_skips_real_dirs_when_symlinks_present() {
        let dir = std::env::temp_dir().join("ace-test-link-skip-real");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        std::fs::create_dir_all(skills.join("my-skill")).expect("create skill dir");
        std::fs::create_dir_all(skills.join("other-skill")).expect("create skill dir");

        let project_skills = project.join(".claude").join("skills");
        std::fs::create_dir_all(project_skills.join("my-skill")).expect("create real dir");
        std::fs::write(project_skills.join("my-skill").join("local.md"), "local override")
            .expect("write local file");

        // Add an existing symlink so this looks like a re-link (not first setup)
        std::os::unix::fs::symlink(
            skills.join("other-skill"),
            project_skills.join("other-skill"),
        )
        .expect("create symlink");

        let result = link_skills(&school, &project).expect("should skip real dirs");
        assert_eq!(result.linked, 0, "both already present");
        assert!(result.moved.is_empty(), "no move when symlinks exist");

        let content = std::fs::read_to_string(project_skills.join("my-skill").join("local.md"))
            .expect("local file should still exist");
        assert_eq!(content, "local override");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn adopt_moves_real_dirs_on_first_setup() {
        let dir = std::env::temp_dir().join("ace-test-link-adopt");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        std::fs::create_dir_all(skills.join("school-skill")).expect("create school skill");

        let project_skills = project.join(".claude").join("skills");
        std::fs::create_dir_all(project_skills.join("my-skill")).expect("create real dir");
        std::fs::write(project_skills.join("my-skill").join("SKILL.md"), "# My Skill")
            .expect("write skill file");

        let result = link_skills(&school, &project).expect("should adopt and link");
        assert_eq!(result.linked, 1, "school skill linked");
        assert_eq!(result.moved, vec!["my-skill"]);

        let prev = project_skills.join("previous-skills").join("my-skill").join("SKILL.md");
        let content = std::fs::read_to_string(prev).expect("moved skill should exist");
        assert_eq!(content, "# My Skill");

        assert!(!project_skills.join("my-skill").exists(), "original should be gone");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn adopt_errors_if_previous_skills_exists() {
        let dir = std::env::temp_dir().join("ace-test-link-adopt-exists");
        let _ = std::fs::remove_dir_all(&dir);

        let school = dir.join("school");
        let project = dir.join("project");
        let skills = school.join("skills");

        std::fs::create_dir_all(skills.join("school-skill")).expect("create school skill");

        let project_skills = project.join(".claude").join("skills");
        std::fs::create_dir_all(project_skills.join("my-skill")).expect("create real dir");
        std::fs::create_dir_all(project_skills.join("previous-skills")).expect("create prev dir");

        let err = link_skills(&school, &project).expect_err("should error");
        let msg = format!("{err}");
        assert!(msg.contains("already exists"), "error: {msg}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Helper to test symlink logic without needing full specifier resolution.
    fn link_skills(school_root: &Path, project_dir: &Path) -> Result<LinkResult, SetupError> {
        let school_skills = school_root.join("skills");
        if !school_skills.exists() {
            return Ok(LinkResult::default());
        }

        let project_skills = project_dir.join(".claude").join("skills");
        std::fs::create_dir_all(&project_skills).map_err(SetupError::WriteConfig)?;

        let moved = adopt_previous_skills(&project_skills)?;

        let mut linked = 0;
        let entries = std::fs::read_dir(&school_skills).map_err(SetupError::WriteConfig)?;

        for entry in entries {
            let entry = entry.map_err(SetupError::WriteConfig)?;
            let file_type = entry.file_type().map_err(SetupError::WriteConfig)?;
            if !file_type.is_dir() {
                continue;
            }

            let skill_name = entry.file_name();
            let link_path = project_skills.join(&skill_name);
            let target = entry.path();

            if link_path.exists() || link_path.symlink_metadata().is_ok() {
                if link_path.symlink_metadata()
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false)
                {
                    let current = std::fs::read_link(&link_path).map_err(SetupError::WriteConfig)?;
                    if current == target {
                        continue;
                    }
                    std::fs::remove_file(&link_path).map_err(SetupError::WriteConfig)?;
                } else {
                    continue;
                }
            }

            std::os::unix::fs::symlink(&target, &link_path).map_err(SetupError::WriteConfig)?;
            linked += 1;
        }

        Ok(LinkResult { linked, moved })
    }
}
