pub mod actions;
pub mod prompt;
pub mod school;
pub mod service;

pub use school::School;
pub use service::Service;

use std::collections::HashMap;
use std::path::Path;

use crate::config;
use crate::config::ace_toml::AceToml;
use crate::config::backend::Backend;
use crate::config::paths::AcePaths;
use crate::config::ConfigError;

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("no config found, ace setup?")]
    NoConfig,
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0}")]
    Path(#[from] crate::config::paths::PathError),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("school: must not be empty")]
    NoSchool,
}

/// Raw config layers, preserved for write-back and inspection.
pub struct ConfigTree {
    pub user: AceToml,
    pub project: AceToml,
    pub local: AceToml,
}

/// Resolved effective state, computed from ConfigTree.
pub struct State {
    pub config: ConfigTree,

    // --- resolved fields ---
    pub school_specifier: Option<String>,
    pub backend: Backend,
    pub session_prompt: String,
    pub env: HashMap<String, String>,
}

impl State {
    pub fn load(project_dir: &Path) -> Result<Self, StateError> {
        let paths = config::paths::resolve(project_dir)?;
        let tree = load_config_tree(&paths)?;
        let resolved = resolve(&tree);
        Ok(Self {
            config: tree,
            school_specifier: resolved.school_specifier,
            backend: resolved.backend,
            session_prompt: resolved.session_prompt,
            env: resolved.env,
        })
    }

    pub fn empty() -> Self {
        Self {
            config: ConfigTree {
                user: AceToml::default(),
                project: AceToml::default(),
                local: AceToml::default(),
            },
            school_specifier: None,
            backend: Backend::Claude,
            session_prompt: String::new(),
            env: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        match &self.school_specifier {
            Some(s) if !s.is_empty() => {}
            _ => return Err(ValidationError::NoSchool),
        }
        Ok(())
    }
}

struct Resolved {
    school_specifier: Option<String>,
    backend: Backend,
    session_prompt: String,
    env: HashMap<String, String>,
}

fn load_config_tree(paths: &AcePaths) -> Result<ConfigTree, StateError> {
    let user = load_or_default(&paths.user)?;
    let project = load_or_default(&paths.project)?;
    let local = load_or_default(&paths.local)?;

    // At least one config must exist on disk
    let any_found = [&paths.user, &paths.project, &paths.local]
        .iter()
        .any(|p| p.exists());
    if !any_found {
        return Err(StateError::NoConfig);
    }

    Ok(ConfigTree { user, project, local })
}

fn load_or_default(path: &Path) -> Result<AceToml, StateError> {
    match config::ace_toml::load(path) {
        Ok(config) => Ok(config),
        Err(ConfigError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(AceToml::default())
        }
        Err(e) => Err(e.into()),
    }
}

/// Resolve effective values from layers. Order: user → project → local (last wins).
fn resolve(tree: &ConfigTree) -> Resolved {
    let layers = [&tree.user, &tree.project, &tree.local];

    // school: last non-empty wins
    let school_specifier = layers
        .iter()
        .rev()
        .find(|l| !l.school.is_empty())
        .map(|l| l.school.clone());

    // backend: last Some wins, fallback Claude
    let backend = layers
        .iter()
        .rev()
        .find_map(|l| l.backend)
        .unwrap_or(Backend::Claude);

    // session_prompt: last non-empty wins
    let session_prompt = layers
        .iter()
        .rev()
        .find(|l| !l.session_prompt.is_empty())
        .map(|l| l.session_prompt.clone())
        .unwrap_or_default();

    // env: additive merge, later keys override
    let mut env = HashMap::new();
    for layer in &layers {
        for (k, v) in &layer.env {
            env.insert(k.clone(), v.clone());
        }
    }

    Resolved {
        school_specifier,
        backend,
        session_prompt,
        env,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ace(school: &str, env: &[(&str, &str)]) -> AceToml {
        AceToml {
            school: school.to_string(),
            backend: None,
            session_prompt: String::new(),
            env: env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }

    fn tree(user: AceToml, project: AceToml, local: AceToml) -> ConfigTree {
        ConfigTree { user, project, local }
    }

    #[test]
    fn resolve_school_last_wins() {
        let t = tree(
            ace("user-school", &[]),
            ace("project-school", &[]),
            ace("", &[]),
        );
        let r = resolve(&t);
        assert_eq!(r.school_specifier.as_deref(), Some("project-school"));
    }

    #[test]
    fn resolve_school_local_overrides() {
        let t = tree(
            ace("user-school", &[]),
            ace("project-school", &[]),
            ace("local-school", &[]),
        );
        let r = resolve(&t);
        assert_eq!(r.school_specifier.as_deref(), Some("local-school"));
    }

    #[test]
    fn resolve_school_none_when_all_empty() {
        let t = tree(ace("", &[]), ace("", &[]), ace("", &[]));
        let r = resolve(&t);
        assert!(r.school_specifier.is_none());
    }

    #[test]
    fn resolve_backend_last_wins() {
        let mut user = ace("", &[]);
        user.backend = Some(Backend::OpenCode);
        let mut project = ace("", &[]);
        project.backend = Some(Backend::Claude);

        let t = tree(user, project, ace("", &[]));
        let r = resolve(&t);
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_backend_fallback_claude() {
        let t = tree(ace("", &[]), ace("", &[]), ace("", &[]));
        let r = resolve(&t);
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_env_additive() {
        let t = tree(
            ace("s", &[("A", "1")]),
            ace("s", &[("B", "2")]),
            ace("s", &[]),
        );
        let r = resolve(&t);
        assert_eq!(r.env["A"], "1");
        assert_eq!(r.env["B"], "2");
    }

    #[test]
    fn resolve_env_override() {
        let t = tree(
            ace("s", &[("KEY", "old"), ("KEEP", "yes")]),
            ace("s", &[("KEY", "new")]),
            ace("s", &[]),
        );
        let r = resolve(&t);
        assert_eq!(r.env["KEY"], "new");
        assert_eq!(r.env["KEEP"], "yes");
    }

    #[test]
    fn resolve_session_prompt_last_wins() {
        let mut user = ace("", &[]);
        user.session_prompt = "user prompt".to_string();
        let mut project = ace("", &[]);
        project.session_prompt = "project prompt".to_string();

        let t = tree(user, project, ace("", &[]));
        let r = resolve(&t);
        assert_eq!(r.session_prompt, "project prompt");
    }

    #[test]
    fn validate_no_school() {
        let state = State::empty();
        assert!(state.validate().is_err());
    }

    #[test]
    fn validate_empty_school() {
        let mut state = State::empty();
        state.school_specifier = Some(String::new());
        assert!(state.validate().is_err());
    }

    #[test]
    fn validate_ok() {
        let mut state = State::empty();
        state.school_specifier = Some("prod9/school".to_string());
        assert!(state.validate().is_ok());
    }
}
