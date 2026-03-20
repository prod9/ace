#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;

pub struct TestEnv {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
        let tmp = tempfile::TempDir::new().expect("create temp dir");
        // Canonicalize to resolve macOS /var → /private/var symlinks.
        let root = tmp.path().canonicalize().expect("canonicalize temp dir");
        Self { _tmp: tmp, root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        assert!(
            !Path::new(rel).is_absolute(),
            "TestEnv::path() rejects absolute paths: {rel}"
        );
        self.root.join(rel)
    }

    pub fn write_file(&self, rel: &str, contents: &str) {
        let path = self.path(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dirs");
        }
        std::fs::write(&path, contents).expect("write file");
    }

    pub fn read_file(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path(rel)).expect("read file")
    }

    pub fn mkdir(&self, rel: &str) {
        std::fs::create_dir_all(self.path(rel)).expect("mkdir");
    }

    pub fn symlink(&self, target: &str, link: &str) {
        let target_path = self.path(target);
        let link_path = self.path(link);
        if let Some(parent) = link_path.parent() {
            std::fs::create_dir_all(parent).expect("create link parent dirs");
        }
        std::os::unix::fs::symlink(&target_path, &link_path).expect("create symlink");
    }

    pub fn git_init(&self) {
        let status = std::process::Command::new("git")
            .args(["init", "--quiet", "--template="])
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .current_dir(&self.root)
            .status()
            .expect("git init");
        assert!(status.success(), "git init failed");
    }

    pub fn assert_exists(&self, rel: &str) {
        let path = self.path(rel);
        assert!(path.exists(), "{} should exist", path.display());
    }

    pub fn assert_not_exists(&self, rel: &str) {
        let path = self.path(rel);
        assert!(!path.exists(), "{} should not exist", path.display());
    }

    pub fn assert_symlink(&self, link: &str, expected_target: &str) {
        let link_path = self.path(link);
        let meta = link_path
            .symlink_metadata()
            .unwrap_or_else(|_| panic!("{} should exist", link_path.display()));
        assert!(meta.file_type().is_symlink(), "{} should be a symlink", link_path.display());

        let actual = std::fs::read_link(&link_path)
            .unwrap_or_else(|_| panic!("read_link {}", link_path.display()));
        let expected = self.path(expected_target);
        assert_eq!(actual, expected, "symlink target mismatch");
    }

    pub fn assert_contains(&self, rel: &str, needle: &str) {
        let content = self.read_file(rel);
        assert!(
            content.contains(needle),
            "{rel} should contain {needle:?}, got:\n{content}"
        );
    }

    pub fn assert_not_contains(&self, rel: &str, needle: &str) {
        let content = self.read_file(rel);
        assert!(
            !content.contains(needle),
            "{rel} should NOT contain {needle:?}, got:\n{content}"
        );
    }

    pub fn git_commit(&self, message: &str) {
        let status = std::process::Command::new("git")
            .args(["add", "-A"])
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .current_dir(&self.root)
            .status()
            .expect("git add");
        assert!(status.success(), "git add failed");

        let status = std::process::Command::new("git")
            .args([
                "-c", "user.email=test@test.com",
                "-c", "user.name=Test",
                "commit", "-m", message, "--allow-empty",
            ])
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .current_dir(&self.root)
            .status()
            .expect("git commit");
        assert!(status.success(), "git commit failed");
    }

    pub fn git_status(&self) -> String {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .current_dir(&self.root)
            .output()
            .expect("git status");
        assert!(output.status.success(), "git status failed");
        String::from_utf8(output.stdout).expect("git status utf8")
    }

    /// Returns an `assert_cmd::Command` for the `ace` binary, pre-configured
    /// with a clean environment and sandbox paths.
    pub fn ace(&self) -> Command {
        let mut cmd = Command::from_std(std::process::Command::new(assert_cmd::cargo_bin!("ace")));
        cmd.env_clear();
        cmd.env("HOME", self.root());
        cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
        cmd.env("XDG_CONFIG_HOME", self.path("config"));
        cmd.env("XDG_CACHE_HOME", self.path("cache"));
        cmd.env("TERM", "dumb");
        cmd.current_dir(self.root());
        cmd
    }
}
