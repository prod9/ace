pub mod io;

use std::path::{Path, PathBuf};

use crate::config;
use crate::config::paths::AcePaths;
use crate::config::school_paths::SchoolPaths;
use crate::config::tree::Tree;
use crate::config::{ConfigError, Scope};
use crate::git::Git;
use crate::state::skills::{Resolved as SkillsResolved, Skills};
use crate::state::{RuntimeOverrides, School, State};

pub use io::{logo, IoError, OutputMode};
use io::Io;

pub struct Ace {
    project_dir: PathBuf,
    tree: Option<Tree>,
    state: Option<State>,
    school: Option<SchoolPaths>,
    school_obj: Option<School>,
    school_obj_loaded: bool,
    skills_obj: Option<Skills<SkillsResolved>>,
    runtime_overrides: RuntimeOverrides,
    scope_override: Option<Scope>,
    io: Io,
    mode: OutputMode,
}

impl Ace {
    pub fn new(project_dir: PathBuf, mode: OutputMode) -> Self {
        Self {
            project_dir,
            tree: None,
            state: None,
            school: None,
            school_obj: None,
            school_obj_loaded: false,
            skills_obj: None,
            runtime_overrides: RuntimeOverrides::default(),
            scope_override: None,
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

    pub fn set_backend_override(&mut self, backend: Option<String>) {
        self.runtime_overrides.backend = backend;
        self.state = None;
    }

    /// Lazy-load the raw config tree (parse-only; no merge, no binding).
    /// Survives `State::resolve` failures, so recovery code paths can still
    /// inspect declared `[[backends]]` after an unknown-backend error.
    pub fn require_tree(&mut self) -> Result<&Tree, ConfigError> {
        if self.tree.is_none() {
            let paths = config::paths::resolve(&self.project_dir)?;
            let mut tree = Tree::load(&paths)?;
            tree.load_school(&self.project_dir)?;
            self.tree = Some(tree);
        }
        Ok(self.tree.as_ref().expect("tree was just set"))
    }

    pub fn set_scope_override(&mut self, scope: Option<Scope>) {
        self.scope_override = scope;
    }

    pub fn scope_override(&self) -> Option<Scope> {
        self.scope_override
    }

    /// Resolve config paths for the current project directory.
    pub fn require_paths(&self) -> Result<AcePaths, ConfigError> {
        config::paths::resolve(&self.project_dir)
    }

    /// Lazy-load tree + school.toml + resolve. No-op if already loaded.
    pub fn require_state(&mut self) -> Result<&State, ConfigError> {
        if self.state.is_none() {
            self.require_tree()?;
            let tree = self.tree.clone().expect("tree just loaded");
            self.state = Some(State::resolve(tree, self.runtime_overrides.clone())?);
        }
        Ok(self.state.as_ref().expect("state was just set"))
    }

    /// Panicking accessor — only after require_state succeeds.
    pub fn state(&self) -> &State {
        self.state.as_ref().expect("state not loaded, call require_state first")
    }

    /// Resolve school paths. Dual context:
    /// - If school.toml exists in project_dir → school repo context
    /// - Otherwise require_tree → specifier → school_paths
    pub fn require_school(&mut self) -> Result<&SchoolPaths, ConfigError> {
        if self.school.is_none() {
            if self.project_dir.join("school.toml").exists() {
                self.school = Some(SchoolPaths {
                    source: ".".to_string(),
                    clone_path: None,
                    root: self.project_dir.clone(),
                });
            } else {
                let tree = self.require_tree()?;
                let Some(spec) = tree.specifier() else {
                    return Err(ConfigError::NoSchool);
                };
                let sp = config::school_paths::resolve(&self.project_dir, &spec)?;
                self.school = Some(sp);
            }
        }
        Ok(self.school.as_ref().expect("school was just confirmed present"))
    }

    /// Re-read school.toml from disk and re-resolve state. Does NOT re-read
    /// ace.toml layers. Also drops cached school bindings so the next
    /// `require_school` / `school()` call re-derives them from the fresh tree.
    pub fn reload_state(&mut self) -> Result<&State, ConfigError> {
        let prev = self.state.as_ref().ok_or(ConfigError::NoConfig)?;
        let mut tree = prev.config.clone();
        tree.load_school(&self.project_dir)?;
        self.tree = Some(tree.clone());
        self.school = None;
        self.school_obj = None;
        self.school_obj_loaded = false;
        self.skills_obj = None;
        self.state = Some(State::resolve(tree, self.runtime_overrides.clone())?);
        Ok(self.state.as_ref().expect("state was just set"))
    }

    /// Lazy-load the resolved School binding. `Ok(None)` when no school is
    /// configured or school.toml is missing/unreadable. Does NOT require the
    /// backend to resolve, so read-only inspection paths still work when the
    /// selector points at an unknown backend.
    pub fn school(&mut self) -> Result<Option<&School>, ConfigError> {
        if !self.school_obj_loaded {
            let tree = self.require_tree()?;
            self.school_obj = tree.school.as_ref().map(|st| School::from(st.clone()));
            self.school_obj_loaded = true;
        }
        Ok(self.school_obj.as_ref())
    }

    /// Lazy-load the resolved SkillSet — discover the school's `skills/` tree
    /// and resolve against the layered config. Errors when no school is
    /// configured (skills require a school root) or discovery I/O fails.
    pub fn skills(&mut self) -> Result<&Skills<SkillsResolved>, ConfigError> {
        if self.skills_obj.is_none() {
            let school_root = self.require_school()?.root.clone();
            let tree = self.require_tree()?.clone();
            let resolved = Skills::discover(&school_root)
                .map_err(ConfigError::Io)?
                .resolve(&tree);
            self.skills_obj = Some(resolved);
        }
        Ok(self.skills_obj.as_ref().expect("skills was just set"))
    }

    pub fn git<'a>(&self, repo: &'a Path) -> Git<'a> {
        Git::new(repo, self.mode)
    }

    // -- output --

    pub fn enter_alt_screen(&self) {
        self.io.enter_alt_screen();
    }

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

    pub fn separator(&mut self) {
        self.io.separator();
    }

    // -- input --

    pub fn prompt_text(&mut self, prompt: &str, initial: Option<&str>) -> Result<String, IoError> {
        self.io.prompt_text(prompt, initial)
    }

    pub fn prompt_confirm(&mut self, prompt: &str, default: bool) -> Result<bool, IoError> {
        self.io.prompt_confirm(prompt, default)
    }

    pub fn prompt_select(&mut self, prompt: &str, options: Vec<String>) -> Result<String, IoError> {
        self.io.prompt_select(prompt, options)
    }
}
