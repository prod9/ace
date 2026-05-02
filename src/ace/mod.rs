pub mod io;

use std::path::{Path, PathBuf};

use crate::backend::{registry, Backend, BackendError};
use crate::config;
use crate::config::ace_toml::AceToml;
use crate::config::paths::AcePaths;
use crate::config::school_paths::SchoolPaths;
use crate::config::tree::Tree;
use crate::config::{ConfigError, Scope};
use crate::git::Git;
use crate::resolver;
use crate::resolver::Resolved;
use crate::school::{School, SchoolError};
use crate::skills::{Decided, SkillError, Skills};

pub use io::{logo, IoError, OutputMode};
use io::Io;

pub struct Ace {
    project_dir: PathBuf,
    tree: Option<Tree>,
    resolved: Option<Resolved>,
    backend: Option<Backend>,
    school_paths: Option<SchoolPaths>,
    /// `None` = not loaded yet. `Some(None)` = loaded, no school configured.
    /// `Some(Some(_))` = loaded, school present.
    school: Option<Option<School>>,
    skills: Option<Skills<Decided>>,
    overrides: AceToml,
    scope_override: Option<Scope>,
    io: Io,
    mode: OutputMode,
}

impl Ace {
    pub fn new(project_dir: PathBuf, mode: OutputMode) -> Self {
        Self {
            project_dir,
            tree: None,
            resolved: None,
            backend: None,
            school_paths: None,
            school: None,
            skills: None,
            overrides: AceToml::default(),
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

    /// Replace the runtime-override layer wholesale. The CLI builds an
    /// `AceToml` from global flags (--backend, --trust, --session-prompt,
    /// --env, ...) and hands it in once at startup. Higher-priority than
    /// any on-disk layer (see `spec/decisions/007.md`).
    pub fn set_overrides(&mut self, overrides: AceToml) {
        self.overrides = overrides;
        self.invalidate_resolved();
    }

    /// Set just the backend field on the override layer. Used by the
    /// PROD9-146 recovery picker when an unknown backend selector is
    /// re-pointed mid-session.
    pub fn override_backend(&mut self, backend: String) {
        self.overrides.backend = Some(backend);
        self.invalidate_resolved();
    }

    pub fn overrides(&self) -> &AceToml {
        &self.overrides
    }

    fn invalidate_resolved(&mut self) {
        self.resolved = None;
        self.backend = None;
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

    /// Lazy-load tree + school.toml + run the merge. Idempotent. The backend
    /// binding is *not* eagerly resolved here — `backend()` does that on
    /// demand so read-only commands survive a stale selector.
    pub fn require_resolved(&mut self) -> Result<&Resolved, ConfigError> {
        if self.resolved.is_none() {
            self.require_tree()?;
            let tree = self.tree.as_ref().expect("tree just loaded");
            self.resolved = Some(resolver::merge(tree, &self.overrides));
        }
        Ok(self.resolved.as_ref().expect("resolved was just set"))
    }

    /// Lazy-load the resolved Backend binding (registry build + name lookup).
    /// `Err(BackendError::Unknown(_))` when the selector points at a name
    /// that isn't a built-in or declared `[[backends]]`.
    pub fn backend(&mut self) -> Result<&Backend, BackendError> {
        if self.backend.is_none() {
            self.require_resolved()?;
            let resolved = self.resolved.as_ref().expect("resolved just loaded");
            self.backend = Some(registry::bind(resolved)?);
        }
        Ok(self.backend.as_ref().expect("backend was just set"))
    }

    /// Resolve school paths. Dual context:
    /// - If school.toml exists in project_dir → school repo context
    /// - Otherwise require_tree → specifier → school_paths
    pub fn require_school(&mut self) -> Result<&SchoolPaths, SchoolError> {
        if self.school_paths.is_none() {
            if self.project_dir.join("school.toml").exists() {
                self.school_paths = Some(SchoolPaths {
                    source: ".".to_string(),
                    clone_path: None,
                    root: self.project_dir.clone(),
                });
            } else {
                let tree = self.require_tree()?;
                let Some(spec) = tree.specifier() else {
                    return Err(SchoolError::Missing);
                };
                let sp = config::school_paths::resolve(&self.project_dir, &spec)?;
                self.school_paths = Some(sp);
            }
        }
        Ok(self.school_paths.as_ref().expect("school_paths was just confirmed present"))
    }

    /// Re-read school.toml from disk and invalidate downstream caches so the
    /// next accessors derive from the freshly loaded tree. Used after
    /// clone-on-first-run.
    pub fn reload_tree(&mut self) -> Result<&Resolved, ConfigError> {
        let mut tree = self.tree.clone().ok_or(ConfigError::NoConfig)?;
        tree.load_school(&self.project_dir)?;
        self.tree = Some(tree);
        self.invalidate_resolved();
        self.school_paths = None;
        self.school = None;
        self.skills = None;
        self.require_resolved()
    }

    /// Lazy-load the resolved School binding. `Ok(None)` when no school is
    /// configured or school.toml is missing/unreadable. Does NOT require the
    /// backend to resolve, so read-only inspection paths still work when the
    /// selector points at an unknown backend.
    pub fn school(&mut self) -> Result<Option<&School>, SchoolError> {
        if self.school.is_none() {
            let tree = self.require_tree()?;
            let school = tree.school.as_ref().map(|st| School::from(st.clone()));
            self.school = Some(school);
        }
        Ok(self.school.as_ref().expect("school just loaded").as_ref())
    }

    /// Lazy-load the resolved SkillSet — discover the school's `skills/` tree
    /// and resolve against the layered config. Errors when no school is
    /// configured (skills require a school root) or discovery I/O fails.
    pub fn skills(&mut self) -> Result<&Skills<Decided>, SkillError> {
        if self.skills.is_none() {
            let school_root = self.require_school()?.root.clone();
            let discovered = Skills::discover(&school_root)?;
            let tree = self.require_tree()?;
            let resolved = discovered.resolve(tree);
            self.skills = Some(resolved);
        }
        Ok(self.skills.as_ref().expect("skills was just set"))
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
