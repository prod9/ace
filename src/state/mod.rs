pub mod actions;
pub mod school;

pub use school::School;

use std::collections::HashMap;

use crate::config::ace_toml::{AceToml, Trust};
use crate::config::backend::Backend;
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
    pub trust: Trust,
}

impl State {
    /// Full resolution: resolve all effective values from config layers.
    /// Tree must have `load_school()` called first if school.toml is needed.
    pub fn resolve(tree: Tree) -> Self {
        let resolved = resolve_layers(&tree);
        let school = tree.school_toml.as_ref().map(|st| School::from(st.clone()));
        Self {
            school_specifier: resolved.school_specifier,
            backend: resolved.backend,
            session_prompt: resolved.session_prompt,
            env: resolved.env,
            trust: resolved.trust,
            school,
            config: tree,
        }
    }

    #[allow(dead_code)] // used in tests + future use
    pub fn empty() -> Self {
        Self {
            config: Tree {
                ace_user: AceToml::default(),
                ace_project: AceToml::default(),
                ace_local: AceToml::default(),
                school_backend: None,
                school_toml: None,
                school_paths: None,
            },
            school_specifier: None,
            backend: Backend::default(),
            session_prompt: String::new(),
            env: HashMap::new(),
            trust: Trust::Default,
            school: None,
        }
    }

    #[allow(dead_code)] // used in tests + future use
    pub fn has_school(&self) -> bool {
        matches!(&self.school_specifier, Some(s) if !s.is_empty())
    }
}

struct Resolved {
    school_specifier: Option<String>,
    backend: Backend,
    session_prompt: String,
    env: HashMap<String, String>,
    trust: Trust,
}

/// Resolve effective values from layers. Order: user → project → local (last wins).
fn resolve_layers(tree: &Tree) -> Resolved {
    let layers = [&tree.ace_user, &tree.ace_project, &tree.ace_local];

    // school: last non-empty wins
    let school_specifier = layers
        .iter()
        .rev()
        .find(|l| !l.school.is_empty())
        .map(|l| l.school.clone());

    // backend: local > project > school > user > fallback Claude
    let backend = tree.ace_local.backend
        .or(tree.ace_project.backend)
        .or(tree.school_backend)
        .or(tree.ace_user.backend)
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

    // trust: local layer only — never from project or school (personal preference).
    // Backcompat: yolo = true in local config is treated as trust = "yolo".
    let trust = if !tree.ace_local.trust.is_default() {
        tree.ace_local.trust
    } else if tree.ace_local.yolo {
        Trust::Yolo
    } else {
        Trust::Default
    };

    Resolved {
        school_specifier,
        backend,
        session_prompt,
        env,
        trust,
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
            trust: Trust::Default,
            yolo: false,
        }
    }

    fn tree(ace_user: AceToml, ace_project: AceToml, ace_local: AceToml) -> Tree {
        Tree {
            ace_user,
            ace_project,
            ace_local,
            school_backend: None,
            school_toml: None,
            school_paths: None,
        }
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
