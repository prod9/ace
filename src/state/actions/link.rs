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
    use std::path::PathBuf;

    struct TestFixture {
        root: PathBuf,
    }

    impl TestFixture {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir().join(name);
            let _ = std::fs::remove_dir_all(&root);

            let fix = Self { root };
            std::fs::create_dir_all(fix.school()).expect("create school dir");
            std::fs::create_dir_all(fix.project()).expect("create project dir");
            fix
        }

        fn school(&self) -> PathBuf { self.root.join("school") }
        fn project(&self) -> PathBuf { self.root.join("project") }
        fn school_skills(&self) -> PathBuf { self.school().join("skills") }
        fn project_skills(&self) -> PathBuf { self.project().join(".claude").join("skills") }

        fn add_school_skill(&self, name: &str) {
            std::fs::create_dir_all(self.school_skills().join(name))
                .expect("create school skill dir");
        }

        fn add_school_skill_with_content(&self, name: &str, file: &str, content: &str) {
            let dir = self.school_skills().join(name);
            std::fs::create_dir_all(&dir).expect("create school skill dir");
            std::fs::write(dir.join(file), content).expect("write skill file");
        }

        fn add_real_skill(&self, name: &str, file: &str, content: &str) {
            let dir = self.project_skills().join(name);
            std::fs::create_dir_all(&dir).expect("create real skill dir");
            std::fs::write(dir.join(file), content).expect("write skill file");
        }

        fn add_symlink(&self, name: &str, target: &Path) {
            let project_skills = self.project_skills();
            std::fs::create_dir_all(&project_skills).expect("mkdir project skills");
            std::os::unix::fs::symlink(target, project_skills.join(name))
                .expect("create symlink");
        }

        fn link(&self) -> Result<LinkResult, SetupError> {
            link_skills(&self.school(), &self.project())
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn link_no_skills_dir() {
        let fix = TestFixture::new("ace-test-link-no-skills");
        let result = fix.link().expect("should succeed");
        assert_eq!(result.linked, 0);
        assert!(result.moved.is_empty());
    }

    #[test]
    fn link_creates_symlinks() {
        let fix = TestFixture::new("ace-test-link-symlinks");
        fix.add_school_skill_with_content("git-commit", "SKILL.md", "# Git Commit");
        fix.add_school_skill_with_content("code-review", "SKILL.md", "# Code Review");

        let result = fix.link().expect("should create symlinks");
        assert_eq!(result.linked, 2);

        let link = fix.project_skills().join("git-commit");
        assert!(link.symlink_metadata().expect("link exists").file_type().is_symlink());

        let content = std::fs::read_to_string(link.join("SKILL.md")).expect("read through symlink");
        assert_eq!(content, "# Git Commit");
    }

    #[test]
    fn link_skips_matching_symlinks() {
        let fix = TestFixture::new("ace-test-link-skip-matching");
        fix.add_school_skill("my-skill");
        fix.add_symlink("my-skill", &fix.school_skills().join("my-skill"));

        let result = fix.link().expect("should skip existing");
        assert_eq!(result.linked, 0);
    }

    #[test]
    fn link_replaces_stale_symlinks() {
        let fix = TestFixture::new("ace-test-link-replace");
        fix.add_school_skill("my-skill");
        fix.add_symlink("my-skill", &fix.root.join("nonexistent"));

        let result = fix.link().expect("should replace stale");
        assert_eq!(result.linked, 1);

        let target = std::fs::read_link(fix.project_skills().join("my-skill")).expect("read link");
        assert_eq!(target, fix.school_skills().join("my-skill"));
    }

    #[test]
    fn link_skips_real_dirs_when_symlinks_present() {
        let fix = TestFixture::new("ace-test-link-skip-real");
        fix.add_school_skill("my-skill");
        fix.add_school_skill("other-skill");
        fix.add_real_skill("my-skill", "local.md", "local override");
        fix.add_symlink("other-skill", &fix.school_skills().join("other-skill"));

        let result = fix.link().expect("should skip real dirs");
        assert_eq!(result.linked, 0, "both already present");
        assert!(result.moved.is_empty(), "no move when symlinks exist");

        let content = std::fs::read_to_string(
            fix.project_skills().join("my-skill").join("local.md"),
        )
        .expect("local file should still exist");
        assert_eq!(content, "local override");
    }

    #[test]
    fn adopt_moves_real_dirs_on_first_setup() {
        let fix = TestFixture::new("ace-test-link-adopt");
        fix.add_school_skill("school-skill");
        fix.add_real_skill("my-skill", "SKILL.md", "# My Skill");

        let result = fix.link().expect("should adopt and link");
        assert_eq!(result.linked, 1, "school skill linked");
        assert_eq!(result.moved, vec!["my-skill"]);

        let prev = fix.project_skills().join("previous-skills").join("my-skill").join("SKILL.md");
        let content = std::fs::read_to_string(prev).expect("moved skill should exist");
        assert_eq!(content, "# My Skill");

        assert!(!fix.project_skills().join("my-skill").exists(), "original should be gone");
    }

    #[test]
    fn adopt_errors_if_previous_skills_exists() {
        let fix = TestFixture::new("ace-test-link-adopt-exists");
        fix.add_school_skill("school-skill");
        fix.add_real_skill("my-skill", "SKILL.md", "");

        let prev = fix.project_skills().join("previous-skills");
        std::fs::create_dir_all(&prev).expect("create prev dir");

        let err = fix.link().expect_err("should error");
        let msg = format!("{err}");
        assert!(msg.contains("already exists"), "error: {msg}");
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

            match link_status(&link_path) {
                LinkStatus::Absent => {}
                LinkStatus::RealDir => continue,
                LinkStatus::Symlink(current) if current == target => continue,
                LinkStatus::Symlink(_) => {
                    std::fs::remove_file(&link_path).map_err(SetupError::WriteConfig)?;
                }
            }

            std::os::unix::fs::symlink(&target, &link_path).map_err(SetupError::WriteConfig)?;
            linked += 1;
        }

        Ok(LinkResult { linked, moved })
    }
}
