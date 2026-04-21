use std::io;
use std::path::Path;

use crate::ace::Ace;
use crate::state::actions::PrepareError;

/// Create a directory-level symlink pointing at `target` at `link`.
/// Platform-split: Unix uses `std::os::unix::fs::symlink`; Windows uses
/// `std::os::windows::fs::symlink_dir` (directory symlinks don't require admin).
fn create_dir_symlink(target: &Path, link: &Path) -> io::Result<()> {
    #[cfg(unix)]
    { std::os::unix::fs::symlink(target, link) }
    #[cfg(windows)]
    { std::os::windows::fs::symlink_dir(target, link) }
}

/// Folders that ACE links from the school clone into the project.
pub const SCHOOL_FOLDERS: &[&str] = &["skills", "rules", "commands", "agents"];

/// Symlink school folders from cache into the project.
pub struct Link<'a> {
    pub school_root: &'a Path,
    pub project_dir: &'a Path,
    pub backend_dir: &'a str,
}

impl Link<'_> {
    pub fn run(&self, _ace: &mut Ace) -> Result<LinkResult, PrepareError> {
        let mut folders = Vec::new();

        for &name in SCHOOL_FOLDERS {
            let school_dir = self.school_root.join(name);
            if !school_dir.exists() {
                continue;
            }

            let project_dir = self.project_dir.join(self.backend_dir).join(name);
            let previous_dir = self.project_dir.join(self.backend_dir).join(format!("previous-{name}"));

            let adopted = adopt_previous(&project_dir, &previous_dir)?;
            let linked = ensure_symlink(&project_dir, &school_dir)?;
            folders.push(FolderResult { name, linked, adopted });
        }

        Ok(LinkResult { folders })
    }
}

#[derive(Debug, Default)]
pub struct LinkResult {
    pub folders: Vec<FolderResult>,
}

#[derive(Debug)]
pub struct FolderResult {
    pub name: &'static str,
    pub linked: bool,
    pub adopted: bool,
}

#[cfg(test)]
impl LinkResult {
    pub fn linked(&self, name: &str) -> bool {
        self.folders.iter().any(|f| f.name == name && f.linked)
    }

    pub fn adopted(&self, name: &str) -> bool {
        self.folders.iter().any(|f| f.name == name && f.adopted)
    }
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

    create_dir_symlink(target, link_path).map_err(PrepareError::Write)?;
    Ok(true)
}

/// On first setup (no symlinks yet), rename the whole dir to `previous-{name}/`.
fn adopt_previous(
    project_dir: &Path,
    previous_dir: &Path,
) -> Result<bool, PrepareError> {
    // Symlink or absent — nothing to adopt.
    if is_symlink(project_dir) || !project_dir.exists() {
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

    std::fs::rename(project_dir, previous_dir).map_err(PrepareError::Write)?;
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
    use tempfile::TempDir;

    struct TestFixture {
        _tmp: TempDir,
        root: PathBuf,
    }

    impl TestFixture {
        fn new() -> Self {
            let tmp = tempfile::tempdir().expect("create tempdir");
            let root = tmp.path().to_path_buf();
            let fix = Self { _tmp: tmp, root };
            std::fs::create_dir_all(fix.school()).expect("create school dir");
            std::fs::create_dir_all(fix.project()).expect("create project dir");
            fix
        }

        fn school(&self) -> PathBuf { self.root.join("school") }
        fn project(&self) -> PathBuf { self.root.join("project") }

        fn school_folder(&self, name: &str) -> PathBuf { self.school().join(name) }
        fn project_folder(&self, name: &str) -> PathBuf {
            self.project().join(".claude").join(name)
        }
        fn previous_folder(&self, name: &str) -> PathBuf {
            self.project().join(".claude").join(format!("previous-{name}"))
        }

        fn add_school_entry(&self, folder: &str, name: &str) {
            std::fs::create_dir_all(self.school_folder(folder).join(name))
                .expect("create school entry dir");
        }

        fn add_school_entry_with_content(
            &self, folder: &str, name: &str, file: &str, content: &str,
        ) {
            let dir = self.school_folder(folder).join(name);
            std::fs::create_dir_all(&dir).expect("create school entry dir");
            std::fs::write(dir.join(file), content).expect("write entry file");
        }

        fn add_real_entry(&self, folder: &str, name: &str, file: &str, content: &str) {
            let dir = self.project_folder(folder).join(name);
            std::fs::create_dir_all(&dir).expect("create real entry dir");
            std::fs::write(dir.join(file), content).expect("write entry file");
        }

        fn link(&self) -> Result<LinkResult, PrepareError> {
            link_all(&self.school(), &self.project())
        }
    }

    #[test]
    fn creates_directory_symlink() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let target = tmp.path().join("target");
        std::fs::create_dir(&target).expect("create target dir");
        let link = tmp.path().join("link");
        create_dir_symlink(&target, &link).expect("create symlink");
        assert!(link.exists(), "symlink must exist");
        assert!(link.is_dir(), "symlink must resolve to directory");
    }

    #[test]
    fn fixtures_are_isolated_per_call() {
        let a = TestFixture::new();
        let b = TestFixture::new();
        assert_ne!(a.root, b.root, "fixtures must be isolated between calls");
    }

    #[test]
    fn link_no_school_folders() {
        let fix = TestFixture::new();
        let result = fix.link().expect("should succeed");
        assert!(!result.linked("skills"));
        assert!(!result.adopted("skills"));
        assert!(result.folders.is_empty());
    }

    #[test]
    fn link_creates_folder_symlink() {
        let fix = TestFixture::new();
        fix.add_school_entry_with_content("skills", "git-commit", "SKILL.md", "# Git Commit");
        fix.add_school_entry_with_content("skills", "code-review", "SKILL.md", "# Code Review");

        let result = fix.link().expect("should create symlink");
        assert!(result.linked("skills"));

        let link = fix.project_folder("skills");
        assert!(link.symlink_metadata().expect("link exists").file_type().is_symlink());

        let target = std::fs::read_link(&link).expect("read link");
        assert_eq!(target, fix.school_folder("skills"));

        let content = std::fs::read_to_string(link.join("git-commit").join("SKILL.md"))
            .expect("read through symlink");
        assert_eq!(content, "# Git Commit");
    }

    #[test]
    fn link_skips_correct_symlink() {
        let fix = TestFixture::new();
        fix.add_school_entry("skills", "my-skill");

        let project_skills = fix.project_folder("skills");
        std::fs::create_dir_all(project_skills.parent().expect("has parent"))
            .expect("mkdir parent");
        create_dir_symlink(&fix.school_folder("skills"), &project_skills)
            .expect("create symlink");

        let result = fix.link().expect("should skip existing");
        assert!(!result.linked("skills"));
    }

    #[test]
    fn link_replaces_stale_symlink() {
        let fix = TestFixture::new();
        fix.add_school_entry("skills", "my-skill");

        let project_skills = fix.project_folder("skills");
        std::fs::create_dir_all(project_skills.parent().expect("has parent"))
            .expect("mkdir parent");
        create_dir_symlink(&fix.root.join("nonexistent"), &project_skills)
            .expect("create stale symlink");

        let result = fix.link().expect("should replace stale");
        assert!(result.linked("skills"));

        let target = std::fs::read_link(&project_skills).expect("read link");
        assert_eq!(target, fix.school_folder("skills"));
    }

    #[test]
    fn link_adopts_empty_dir() {
        let fix = TestFixture::new();
        fix.add_school_entry("skills", "my-skill");

        std::fs::create_dir_all(fix.project_folder("skills")).expect("mkdir");

        let result = fix.link().expect("should adopt and link");
        assert!(result.adopted("skills"));
        assert!(result.linked("skills"));

        let target = std::fs::read_link(fix.project_folder("skills")).expect("read link");
        assert_eq!(target, fix.school_folder("skills"));
        assert!(fix.previous_folder("skills").exists());
    }

    #[test]
    fn adopt_renames_dir_on_first_setup() {
        let fix = TestFixture::new();
        fix.add_school_entry("skills", "school-skill");
        fix.add_real_entry("skills", "my-skill", "SKILL.md", "# My Skill");

        let result = fix.link().expect("should adopt and link");
        assert!(result.linked("skills"), "school skills linked");
        assert!(result.adopted("skills"), "skills dir adopted");

        let prev = fix.previous_folder("skills").join("my-skill").join("SKILL.md");
        let content = std::fs::read_to_string(prev).expect("moved skill should exist");
        assert_eq!(content, "# My Skill");

        assert!(
            fix.project_folder("skills").symlink_metadata().expect("exists").file_type().is_symlink(),
            "project skills should be a symlink"
        );
    }

    #[test]
    fn adopt_errors_if_previous_exists() {
        let fix = TestFixture::new();
        fix.add_school_entry("skills", "school-skill");
        fix.add_real_entry("skills", "my-skill", "SKILL.md", "");

        std::fs::create_dir_all(fix.previous_folder("skills")).expect("create prev dir");

        let err = fix.link().expect_err("should error");
        let msg = format!("{err}");
        assert!(msg.contains("already exists"), "error: {msg}");
    }

    #[test]
    fn adopt_skips_when_already_symlinked() {
        let fix = TestFixture::new();
        fix.add_school_entry("skills", "my-skill");

        let project_skills = fix.project_folder("skills");
        std::fs::create_dir_all(project_skills.parent().expect("has parent"))
            .expect("mkdir parent");
        create_dir_symlink(&fix.school_folder("skills"), &project_skills)
            .expect("create symlink");

        let result = fix.link().expect("should succeed");
        assert!(!result.adopted("skills"), "no adoption when already symlinked");
    }

    #[test]
    fn link_all_four_folders() {
        let fix = TestFixture::new();
        for folder in SCHOOL_FOLDERS {
            fix.add_school_entry(folder, "test-entry");
        }

        let result = fix.link().expect("should link all");
        assert_eq!(result.folders.len(), 4);
        for folder in SCHOOL_FOLDERS {
            assert!(result.linked(folder), "{folder} should be linked");
        }
    }

    #[test]
    fn link_skips_absent_folders() {
        let fix = TestFixture::new();
        fix.add_school_entry("skills", "a-skill");
        fix.add_school_entry("commands", "a-command");

        let result = fix.link().expect("should link partial");
        assert_eq!(result.folders.len(), 2);
        assert!(result.linked("skills"));
        assert!(result.linked("commands"));
        assert!(!result.linked("rules"));
        assert!(!result.linked("agents"));
    }

    /// Helper — exercises the same loop as Link::run without needing an Ace instance.
    fn link_all(school_root: &Path, project_dir: &Path) -> Result<LinkResult, PrepareError> {
        let mut folders = Vec::new();

        for &name in SCHOOL_FOLDERS {
            let school_dir = school_root.join(name);
            if !school_dir.exists() {
                continue;
            }

            let proj_dir = project_dir.join(".claude").join(name);
            let prev_dir = project_dir.join(".claude").join(format!("previous-{name}"));

            let adopted = adopt_previous(&proj_dir, &prev_dir)?;
            let linked = ensure_symlink(&proj_dir, &school_dir)?;
            folders.push(FolderResult { name, linked, adopted });
        }

        Ok(LinkResult { folders })
    }
}
