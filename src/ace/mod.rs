use std::path::{Path, PathBuf};

use crate::config;
use crate::config::school_paths::SchoolPaths;
use crate::config::tree::Tree;
use crate::config::ConfigError;
use crate::events::OutputMode;
use crate::git::Git;
use crate::state::State;
use crate::term_ui::sink::EventSink;

pub struct Ace {
    project_dir: PathBuf,
    state: Option<State>,
    tree: Option<Tree>,
    school: Option<SchoolPaths>,
    sink: EventSink,
    mode: OutputMode,
}

impl Ace {
    pub fn new(project_dir: PathBuf, mode: OutputMode) -> Self {
        Self {
            project_dir,
            state: None,
            tree: None,
            school: None,
            sink: EventSink::new(mode),
            mode,
        }
    }

    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    /// Lazy-load tree + school.toml + resolve. No-op if already loaded.
    pub fn require_state(&mut self) -> Result<&State, ConfigError> {
        if self.state.is_none() {
            let paths = config::paths::resolve(&self.project_dir)?;
            let mut tree = config::tree::Tree::load(&paths)?;

            let mut school_toml_opt = None;
            if let Some(spec) = State::resolve_specifier(&tree) {
                let sp = config::school_paths::resolve(&self.project_dir, &spec)?;
                let school_toml_path = sp.root.join("school.toml");
                if school_toml_path.exists() {
                    if let Ok(school) = config::school_toml::load(&school_toml_path) {
                        tree.school_backend = school.backend;
                        school_toml_opt = Some(school);
                    }
                }
                self.school = Some(sp);
            }

            self.tree = Some(tree.clone());
            self.state = Some(State::resolve(tree, school_toml_opt));
        }
        Ok(self.state.as_ref().unwrap())
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
        Ok(self.school.as_ref().unwrap())
    }

    /// Re-read school.toml, feed school_backend, re-resolve from stored tree.
    /// Does NOT re-read ace.toml. Also refreshes cached school paths.
    pub fn reload_state(&mut self) -> Result<&State, ConfigError> {
        let mut tree = self.tree.clone().ok_or(ConfigError::NoConfig)?;

        let mut school_toml_opt = None;
        if let Some(spec) = State::resolve_specifier(&tree) {
            let sp = config::school_paths::resolve(&self.project_dir, &spec)?;
            let school_toml_path = sp.root.join("school.toml");
            if school_toml_path.exists() {
                if let Ok(school) = config::school_toml::load(&school_toml_path) {
                    tree.school_backend = school.backend;
                    school_toml_opt = Some(school);
                }
            }
            self.school = Some(sp);
        }

        self.tree = Some(tree.clone());
        self.state = Some(State::resolve(tree, school_toml_opt));
        Ok(self.state.as_ref().unwrap())
    }

    pub fn output_mode(&self) -> OutputMode {
        self.mode
    }

    pub fn git<'a>(&self, repo: &'a Path) -> Git<'a> {
        Git::new(repo, self.mode)
    }

    pub fn progress(&mut self, msg: &str) {
        self.sink.progress(msg);
    }

    pub fn done(&mut self, msg: &str) {
        self.sink.done(msg);
    }

    pub fn warn(&mut self, msg: &str) {
        self.sink.warn(msg);
    }

    pub fn error(&mut self, msg: &str) {
        self.sink.error(msg);
    }

    pub fn data(&mut self, msg: &str) {
        self.sink.data(msg);
    }
}
