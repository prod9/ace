use std::path::Path;

use crate::ace::Ace;
use super::prepare::PrepareError;

const PREVIOUS_SKILLS_DIR: &str = "previous-skills";
const PREVIOUS_RULES_DIR: &str = "previous-rules";

/// Symlink school skills folder from cache into the project.
pub struct Link<'a> {
    pub school_root: &'a Path,
    pub project_dir: &'a Path,
    pub skills_dir: &'a str, // TODO: This should not be named skills_dir, it's .claude we need to
                             // .join("skills") again
}

impl Link<'_> {
    pub fn run(&self, _ace: &mut Ace) -> Result<LinkResult, PrepareError> {
        let school_skills = self.school_root.join("skills");
        let school_rules = self.school_root.join("rules");
        if !school_skills.exists() && !school_rules.exists() {
            return Ok(LinkResult::default());
        }

        let mut result = LinkResult::default();
        if school_skills.exists() {
            let project_skills = self.project_dir.join(self.skills_dir).join("skills");
            let previous_skills_dir = self.project_dir.join(self.skills_dir).join(PREVIOUS_SKILLS_DIR);
            result.skills_adopted = adopt_previous_skills(&project_skills, &previous_skills_dir)?;
            result.skills_linked = ensure_symlink(&project_skills, &school_skills)?;
        }

        if school_rules.exists() {
            let project_rules = self.project_dir.join(self.skills_dir).join("rules");
            let previous_rules_dir = self.project_dir.join(self.skills_dir).join(PREVIOUS_RULES_DIR);
            result.rules_adopted = adopt_previous_skills(&project_rules, &previous_rules_dir)?;
            result.rules_linked =  ensure_symlink(&project_rules, &school_rules)?;
        }

        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct LinkResult {
    pub skills_linked: bool,
    pub skills_adopted: bool,
    pub rules_linked: bool,
    pub rules_adopted: bool,
}

/// Create or update the folder-level symlink from `link_path` to `target`.
fn ensure_symlink(link_path: &Path, target: &Path) -> Result<bool, PrepareError> {
    match link_status(link_path) {
        LinkStatus::Symlink(current) if current == target => return Ok(false),
        LinkStatus::Symlink(_) => {
            std::fs::remove_file(link_path).map_err(PrepareError::Write)?;
        }
        LinkStatus::RealDir => {
            return Err(PrepareError::Write(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "{} is a real directory — adoption should have renamed it away",
                    link_path.display()
                ),
            )));
        }
        LinkStatus::Absent => {
            if let Some(parent) = link_path.parent() {
                std::fs::create_dir_all(parent).map_err(PrepareError::Write)?;
            }
        }
    }

    std::os::unix::fs::symlink(target, link_path).map_err(PrepareError::Write)?;
    Ok(true)
}

/// On first setup (no symlinks yet), rename the whole skills dir to `previous-skills/`.
fn adopt_previous_skills(
    project_skills: &Path,
    previous_dir: &Path,
) -> Result<bool, PrepareError> {
    // Symlink or absent — nothing to adopt.
    if is_symlink(project_skills) || !project_skills.exists() {
        return Ok(false);
    }

    // It's a real directory — rename it wholesale.
    if previous_dir.exists() {
        return Err(PrepareError::Write(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists — consolidate or remove it first",
                previous_dir.display()
            ),
        )));
    }

    std::fs::rename(project_skills, previous_dir).map_err(PrepareError::Write)?;
    Ok(true)
}

fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

enum LinkStatus {
    Absent,
    Symlink(std::path::PathBuf),
    RealDir,
}

fn link_status(path: &Path) -> LinkStatus {
    if is_symlink(path) {
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
        fn previous_skills(&self) -> PathBuf { self.project().join(".claude").join("previous-skills") }

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

        fn link(&self) -> Result<LinkResult, PrepareError> {
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
        assert!(!result.linked);
        assert!(!result.adopted);
    }

    #[test]
    fn link_creates_folder_symlink() {
        let fix = TestFixture::new("ace-test-link-folder");
        fix.add_school_skill_with_content("git-commit", "SKILL.md", "# Git Commit");
        fix.add_school_skill_with_content("code-review", "SKILL.md", "# Code Review");

        let result = fix.link().expect("should create symlink");
        assert!(result.linked);

        let link = fix.project_skills();
        assert!(link.symlink_metadata().expect("link exists").file_type().is_symlink());

        let target = std::fs::read_link(&link).expect("read link");
        assert_eq!(target, fix.school_skills());

        let content = std::fs::read_to_string(link.join("git-commit").join("SKILL.md"))
            .expect("read through symlink");
        assert_eq!(content, "# Git Commit");
    }

    #[test]
    fn link_skips_correct_symlink() {
        let fix = TestFixture::new("ace-test-link-skip-correct");
        fix.add_school_skill("my-skill");

        // Create correct symlink manually
        let project_skills = fix.project_skills();
        std::fs::create_dir_all(project_skills.parent().expect("has parent"))
            .expect("mkdir parent");
        std::os::unix::fs::symlink(fix.school_skills(), &project_skills)
            .expect("create symlink");

        let result = fix.link().expect("should skip existing");
        assert!(!result.linked);
    }

    #[test]
    fn link_replaces_stale_symlink() {
        let fix = TestFixture::new("ace-test-link-replace-stale");
        fix.add_school_skill("my-skill");

        // Create stale symlink
        let project_skills = fix.project_skills();
        std::fs::create_dir_all(project_skills.parent().expect("has parent"))
            .expect("mkdir parent");
        std::os::unix::fs::symlink(fix.root.join("nonexistent"), &project_skills)
            .expect("create stale symlink");

        let result = fix.link().expect("should replace stale");
        assert!(result.linked);

        let target = std::fs::read_link(&project_skills).expect("read link");
        assert_eq!(target, fix.school_skills());
    }

    #[test]
    fn link_adopts_empty_dir() {
        let fix = TestFixture::new("ace-test-link-replace-empty");
        fix.add_school_skill("my-skill");

        // Create empty project skills dir — adoption renames it away, then symlink is created
        std::fs::create_dir_all(fix.project_skills()).expect("mkdir");

        let result = fix.link().expect("should adopt and link");
        assert!(result.adopted);
        assert!(result.linked);

        let target = std::fs::read_link(fix.project_skills()).expect("read link");
        assert_eq!(target, fix.school_skills());

        // Empty dir was renamed to previous-skills/
        assert!(fix.previous_skills().exists());
    }

    #[test]
    fn adopt_renames_dir_on_first_setup() {
        let fix = TestFixture::new("ace-test-link-adopt");
        fix.add_school_skill("school-skill");
        fix.add_real_skill("my-skill", "SKILL.md", "# My Skill");

        let result = fix.link().expect("should adopt and link");
        assert!(result.linked, "school skills linked");
        assert!(result.adopted, "skills dir adopted");

        // Entire dir was renamed — skill content preserved inside previous-skills/
        let prev = fix.previous_skills().join("my-skill").join("SKILL.md");
        let content = std::fs::read_to_string(prev).expect("moved skill should exist");
        assert_eq!(content, "# My Skill");

        // project_skills is now a symlink
        assert!(
            fix.project_skills().symlink_metadata().expect("exists").file_type().is_symlink(),
            "project skills should be a symlink"
        );
    }

    #[test]
    fn adopt_errors_if_previous_skills_exists() {
        let fix = TestFixture::new("ace-test-link-adopt-exists");
        fix.add_school_skill("school-skill");
        fix.add_real_skill("my-skill", "SKILL.md", "");

        // Create previous-skills at sibling level
        std::fs::create_dir_all(fix.previous_skills()).expect("create prev dir");

        let err = fix.link().expect_err("should error");
        let msg = format!("{err}");
        assert!(msg.contains("already exists"), "error: {msg}");
    }

    #[test]
    fn adopt_skips_when_already_symlinked() {
        let fix = TestFixture::new("ace-test-link-adopt-skip");
        fix.add_school_skill("my-skill");

        // Already a symlink — adoption should be skipped
        let project_skills = fix.project_skills();
        std::fs::create_dir_all(project_skills.parent().expect("has parent"))
            .expect("mkdir parent");
        std::os::unix::fs::symlink(fix.school_skills(), &project_skills)
            .expect("create symlink");

        let result = fix.link().expect("should succeed");
        assert!(!result.adopted, "no adoption when already symlinked");
    }

    /// Helper to test symlink logic without needing full specifier resolution.
    fn link_skills(school_root: &Path, project_dir: &Path) -> Result<LinkResult, PrepareError> {
        let school_skills = school_root.join("skills");
        if !school_skills.exists() {
            return Ok(LinkResult::default());
        }

        let project_skills = project_dir.join(".claude").join("skills");
        let previous_skills_dir = project_dir.join(".claude").join(PREVIOUS_SKILLS_DIR);

        let adopted = adopt_previous_skills(&project_skills, &previous_skills_dir)?;
        let linked = ensure_symlink(&project_skills, &school_skills)?;

        Ok(LinkResult { linked, adopted })
    }
}
