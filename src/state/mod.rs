pub mod actions;
pub mod school;
pub mod service;

pub use school::School;

use std::collections::HashMap;

use crate::config::ace_toml::AceToml;
use crate::config::backend::Backend;
use crate::config::school_toml::SchoolToml;
use crate::config::tree::Tree;

/// Resolved effective state, computed from config::Tree.
pub struct State {
    pub config: Tree,

    // --- resolved fields ---
    pub school_specifier: Option<String>,
    pub backend: Backend,
    pub session_prompt: String,
    pub env: HashMap<String, String>,
    pub school: Option<School>,
}

impl State {
    /// First pass: resolve school specifier from ace.toml layers only.
    /// Call this before loading school.toml to know which school to load.
    pub fn resolve_specifier(tree: &Tree) -> Option<String> {
        if !tree.local.school.is_empty() {
            Some(tree.local.school.clone())
        } else if !tree.project.school.is_empty() {
            Some(tree.project.school.clone())
        } else if !tree.user.school.is_empty() {
            Some(tree.user.school.clone())
        } else {
            None
        }
    }

    /// Full resolution: resolve all effective values from config layers.
    /// Set tree.school_backend before calling if school.toml is available.
    pub fn resolve(tree: Tree, school_toml: Option<SchoolToml>) -> Self {
        let resolved = resolve_layers(&tree);
        Self {
            config: tree,
            school_specifier: resolved.school_specifier,
            backend: resolved.backend,
            session_prompt: resolved.session_prompt,
            env: resolved.env,
            school: school_toml.map(School::from),
        }
    }

    pub fn empty() -> Self {
        Self {
            config: Tree {
                user: AceToml::default(),
                project: AceToml::default(),
                local: AceToml::default(),
                school_backend: None,
            },
            school_specifier: None,
            backend: Backend::default(),
            session_prompt: String::new(),
            env: HashMap::new(),
            school: None,
        }
    }

    pub fn has_school(&self) -> bool {
        matches!(&self.school_specifier, Some(s) if !s.is_empty())
    }
}

struct Resolved {
    school_specifier: Option<String>,
    backend: Backend,
    session_prompt: String,
    env: HashMap<String, String>,
}

/// Resolve effective values from layers. Order: user → project → local (last wins).
fn resolve_layers(tree: &Tree) -> Resolved {
    let layers = [&tree.user, &tree.project, &tree.local];

    // school: last non-empty wins
    let school_specifier = layers
        .iter()
        .rev()
        .find(|l| !l.school.is_empty())
        .map(|l| l.school.clone());

    // backend: local > project > school > user > fallback Claude
    let backend = tree.local.backend
        .or(tree.project.backend)
        .or(tree.school_backend)
        .or(tree.user.backend)
        .unwrap_or_default();

    // session_prompt: last Some wins (Some("") is a valid override to empty)
    let session_prompt = layers
        .iter()
        .rev()
        .find_map(|l| l.session_prompt.clone())
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
            session_prompt: None,
            env: env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }

    fn tree(user: AceToml, project: AceToml, local: AceToml) -> Tree {
        Tree { user, project, local, school_backend: None }
    }

    #[test]
    fn resolve_school_last_wins() {
        let t = tree(
            ace("user-school", &[]),
            ace("project-school", &[]),
            ace("", &[]),
        );
        let r = resolve_layers(&t);
        assert_eq!(r.school_specifier.as_deref(), Some("project-school"));
    }

    #[test]
    fn resolve_school_local_overrides() {
        let t = tree(
            ace("user-school", &[]),
            ace("project-school", &[]),
            ace("local-school", &[]),
        );
        let r = resolve_layers(&t);
        assert_eq!(r.school_specifier.as_deref(), Some("local-school"));
    }

    #[test]
    fn resolve_school_none_when_all_empty() {
        let t = tree(ace("", &[]), ace("", &[]), ace("", &[]));
        let r = resolve_layers(&t);
        assert!(r.school_specifier.is_none());
    }

    #[test]
    fn resolve_backend_last_wins() {
        let mut user = ace("", &[]);
        user.backend = Some(Backend::OpenCode);
        let mut project = ace("", &[]);
        project.backend = Some(Backend::Claude);

        let t = tree(user, project, ace("", &[]));
        let r = resolve_layers(&t);
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_backend_fallback_claude() {
        let t = tree(ace("", &[]), ace("", &[]), ace("", &[]));
        let r = resolve_layers(&t);
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_env_additive() {
        let t = tree(
            ace("s", &[("A", "1")]),
            ace("s", &[("B", "2")]),
            ace("s", &[]),
        );
        let r = resolve_layers(&t);
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
        let r = resolve_layers(&t);
        assert_eq!(r.env["KEY"], "new");
        assert_eq!(r.env["KEEP"], "yes");
    }

    #[test]
    fn resolve_session_prompt_last_wins() {
        let mut user = ace("", &[]);
        user.session_prompt = Some("user prompt".to_string());
        let mut project = ace("", &[]);
        project.session_prompt = Some("project prompt".to_string());

        let t = tree(user, project, ace("", &[]));
        let r = resolve_layers(&t);
        assert_eq!(r.session_prompt, "project prompt");
    }

    #[test]
    fn resolve_backend_school_toml_used() {
        let mut t = tree(ace("", &[]), ace("", &[]), ace("", &[]));
        t.school_backend = Some(Backend::OpenCode);

        let r = resolve_layers(&t);
        assert_eq!(r.backend, Backend::OpenCode);
    }

    #[test]
    fn resolve_backend_project_overrides_school() {
        let mut project = ace("", &[]);
        project.backend = Some(Backend::Claude);

        let mut t = tree(ace("", &[]), project, ace("", &[]));
        t.school_backend = Some(Backend::OpenCode);

        let r = resolve_layers(&t);
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_backend_local_overrides_school() {
        let mut local = ace("", &[]);
        local.backend = Some(Backend::Claude);

        let mut t = tree(ace("", &[]), ace("", &[]), local);
        t.school_backend = Some(Backend::OpenCode);

        let r = resolve_layers(&t);
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_backend_school_overrides_user() {
        let mut user = ace("", &[]);
        user.backend = Some(Backend::Claude);

        let mut t = tree(user, ace("", &[]), ace("", &[]));
        t.school_backend = Some(Backend::OpenCode);

        let r = resolve_layers(&t);
        assert_eq!(r.backend, Backend::OpenCode);
    }

    #[test]
    fn has_school_none() {
        let state = State::empty();
        assert!(!state.has_school());
    }

    #[test]
    fn has_school_empty() {
        let mut state = State::empty();
        state.school_specifier = Some(String::new());
        assert!(!state.has_school());
    }

    #[test]
    fn has_school_ok() {
        let mut state = State::empty();
        state.school_specifier = Some("prod9/school".to_string());
        assert!(state.has_school());
    }
}
