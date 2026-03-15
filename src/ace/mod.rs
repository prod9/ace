pub mod io;

use std::path::{Path, PathBuf};

use crate::config;
use crate::config::school_paths::SchoolPaths;
use crate::config::tree::Tree;
use crate::config::ConfigError;
use crate::git::Git;
use crate::state::State;

pub use io::{logo, IoError, OutputMode};
use io::Io;

pub struct Ace {
    project_dir: PathBuf,
    state: Option<State>,
    school: Option<SchoolPaths>,
    io: Io,
    mode: OutputMode,
}

impl Ace {
    pub fn new(project_dir: PathBuf, mode: OutputMode) -> Self {
        Self {
            project_dir,
            state: None,
            school: None,
            io: Io::new(mode),
            mode,
        }
    }

    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    pub fn mode(&self) -> OutputMode {
        self.mode
    }

    /// Lazy-load tree + school.toml + resolve. No-op if already loaded.
    pub fn require_state(&mut self) -> Result<&State, ConfigError> {
        if self.state.is_none() {
            let paths = config::paths::resolve(&self.project_dir)?;
            let mut tree = Tree::load(&paths)?;
            tree.load_school(&self.project_dir)?;
            self.school = tree.school_paths.take();
            self.state = Some(State::resolve(tree));
        }
        Ok(self.state.as_ref().expect("state was just set"))
    }

    /// Panicking accessor — only after require_state succeeds.
    pub fn state(&self) -> &State {
        self.state.as_ref().expect("state not loaded, call require_state first")
    }

    /// Resolve school paths. Dual context:
    /// - If school.toml exists in project_dir → school repo context
    /// - Otherwise require_state → specifier → school_paths
    pub fn require_school(&mut self) -> Result<&SchoolPaths, ConfigError> {
        if self.school.is_none() {
            // Direct school repo context
            if self.project_dir.join("school.toml").exists() {
                self.school = Some(SchoolPaths {
                    source: ".".to_string(),
                    cache: None,
                    root: self.project_dir.clone(),
                });
            } else {
                // Load state to get specifier
                self.require_state()?;
                if self.school.is_none() {
                    return Err(ConfigError::NoSchool);
                }
            }
        }
        Ok(self.school.as_ref().expect("school was just confirmed present"))
    }

    /// Re-read school.toml, feed school_backend, re-resolve from stored tree.
    /// Does NOT re-read ace.toml. Also refreshes cached school paths.
    pub fn reload_state(&mut self) -> Result<&State, ConfigError> {
        let prev = self.state.as_ref().ok_or(ConfigError::NoConfig)?;
        let mut tree = prev.config.clone();
        tree.load_school(&self.project_dir)?;
        self.school = tree.school_paths.take();
        self.state = Some(State::resolve(tree));
        Ok(self.state.as_ref().expect("state was just set"))
    }

    pub fn git<'a>(&self, repo: &'a Path) -> Git<'a> {
        Git::new(repo, self.mode)
    }

    // -- output --

    pub fn progress(&mut self, msg: &str) {
        self.io.progress(msg);
    }

    pub fn done(&mut self, msg: &str) {
        self.io.done(msg);
    }

    pub fn warn(&mut self, msg: &str) {
        self.io.warn(msg);
    }

    pub fn error(&mut self, msg: &str) {
        self.io.error(msg);
    }

    pub fn hint(&mut self, msg: &str) {
        self.io.hint(msg);
    }

    pub fn data(&mut self, msg: &str) {
        self.io.data(msg);
    }

    // -- input --

    pub fn prompt_text(&mut self, prompt: &str, initial: Option<&str>) -> Result<String, IoError> {
        self.io.prompt_text(prompt, initial)
    }

    pub fn prompt_select(&mut self, prompt: &str, options: Vec<String>) -> Result<String, IoError> {
        self.io.prompt_select(prompt, options)
    }
}
