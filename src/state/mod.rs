pub mod discover;
pub mod resolver;
pub mod school;
pub mod skill_set;

pub use school::School;

use std::collections::HashMap;

use crate::config::ace_toml::{AceToml, Trust};
use crate::config::backend::Backend;
use crate::config::tree::Tree;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeOverrides {
    pub backend: Option<Backend>,
}

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
    pub resume: bool,
    pub skip_update: bool,
}

impl State {
    /// Full resolution: resolve all effective values from config layers.
    /// Tree must have `load_school()` called first if school.toml is needed.
    pub fn resolve(tree: Tree, overrides: RuntimeOverrides) -> Self {
        let resolved = resolve_layers(&tree, overrides);
        let school = tree.school_toml.as_ref().map(|st| School::from(st.clone()));
        Self {
            school_specifier: resolved.school_specifier,
            backend: resolved.backend,
            session_prompt: resolved.session_prompt,
            env: resolved.env,
            trust: resolved.trust,
            resume: resolved.resume,
            skip_update: resolved.skip_update,
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
            resume: true,
            skip_update: false,
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
    resume: bool,
    skip_update: bool,
}

/// Resolve effective values from layers. Order: user → project → local (last wins).
fn resolve_layers(tree: &Tree, overrides: RuntimeOverrides) -> Resolved {
    let layers = [&tree.ace_user, &tree.ace_project, &tree.ace_local];

    // school: last non-empty wins
    let school_specifier = layers
        .iter()
        .rev()
        .find(|l| !l.school.is_empty())
        .map(|l| l.school.clone());

    // backend: local > project > user > school > fallback Claude
    let backend = overrides.backend
        .or(tree.ace_local.backend)
        .or(tree.ace_project.backend)
        .or(tree.ace_user.backend)
        .or(tree.school_backend)
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

    // trust: user + local only — never from project or school (personal preference).
    // Local wins over user. Backcompat: yolo = true treated as trust = "yolo".
    let trust = if !tree.ace_local.trust.is_default() {
        tree.ace_local.trust
    } else if tree.ace_local.yolo {
        Trust::Yolo
    } else if !tree.ace_user.trust.is_default() {
        tree.ace_user.trust
    } else if tree.ace_user.yolo {
        Trust::Yolo
    } else {
        Trust::Default
    };

    // resume: user + local only (personal preference). Local wins over user. Default true.
    let resume = tree.ace_local.resume
        .or(tree.ace_user.resume)
        .unwrap_or(true);

    // skip_update: standard three-layer, last Some wins. Default false.
    let skip_update = layers
        .iter()
        .rev()
        .find_map(|l| l.skip_update)
        .unwrap_or(false);

    Resolved {
        school_specifier,
        backend,
        session_prompt,
        env,
        trust,
        resume,
        skip_update,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ace(school: &str, env: &[(&str, &str)]) -> AceToml {
        AceToml {
            school: school.to_string(),
            env: env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            ..AceToml::default()
        }
    }

    fn tree(ace_project: AceToml, ace_local: AceToml) -> Tree {
        Tree {
            ace_user: AceToml::default(),
            ace_project,
            ace_local,
            school_backend: None,
            school_toml: None,
            school_paths: None,
        }
    }

    #[test]
    fn resolve_school_project_wins() {
        let t = tree(
            ace("project-school", &[]),
            ace("", &[]),
        );
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.school_specifier.as_deref(), Some("project-school"));
    }

    #[test]
    fn resolve_school_local_overrides() {
        let t = tree(
            ace("project-school", &[]),
            ace("local-school", &[]),
        );
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.school_specifier.as_deref(), Some("local-school"));
    }

    #[test]
    fn resolve_school_none_when_all_empty() {
        let t = tree(ace("", &[]), ace("", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(r.school_specifier.is_none());
    }

    #[test]
    fn resolve_backend_local_overrides_project() {
        let mut project = ace("", &[]);
        project.backend = Some(Backend::Codex);
        let mut local = ace("", &[]);
        local.backend = Some(Backend::Claude);

        let t = tree(project, local);
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_backend_fallback_claude() {
        let t = tree(ace("", &[]), ace("", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_env_additive() {
        let t = tree(
            ace("s", &[("A", "1")]),
            ace("s", &[("B", "2")]),
        );
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.env["A"], "1");
        assert_eq!(r.env["B"], "2");
    }

    #[test]
    fn resolve_env_override() {
        let t = tree(
            ace("s", &[("KEY", "old"), ("KEEP", "yes")]),
            ace("s", &[("KEY", "new")]),
        );
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.env["KEY"], "new");
        assert_eq!(r.env["KEEP"], "yes");
    }

    #[test]
    fn resolve_session_prompt_last_wins() {
        let mut project = ace("", &[]);
        project.session_prompt = Some("project prompt".to_string());

        let t = tree(project, ace("", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.session_prompt, "project prompt");
    }

    #[test]
    fn resolve_backend_school_toml_used() {
        let mut t = tree(ace("", &[]), ace("", &[]));
        t.school_backend = Some(Backend::Codex);

        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.backend, Backend::Codex);
    }

    #[test]
    fn resolve_backend_project_overrides_school() {
        let mut project = ace("", &[]);
        project.backend = Some(Backend::Claude);

        let mut t = tree(project, ace("", &[]));
        t.school_backend = Some(Backend::Codex);

        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_backend_local_overrides_school() {
        let mut local = ace("", &[]);
        local.backend = Some(Backend::Claude);

        let mut t = tree(ace("", &[]), local);
        t.school_backend = Some(Backend::Codex);

        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert_eq!(r.backend, Backend::Claude);
    }

    #[test]
    fn resolve_backend_override_beats_all_layers() {
        let mut project = ace("", &[]);
        project.backend = Some(Backend::Flaude);

        let mut local = ace("", &[]);
        local.backend = Some(Backend::Claude);

        let mut t = tree(project, local);
        t.school_backend = Some(Backend::Codex);

        let r = resolve_layers(
            &t,
            RuntimeOverrides {
                backend: Some(Backend::Codex),
            },
        );
        assert_eq!(r.backend, Backend::Codex);
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

    #[test]
    fn skip_update_defaults_false() {
        let t = tree(ace("s", &[]), ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(!r.skip_update);
    }

    #[test]
    fn skip_update_project_true() {
        let mut project = ace("s", &[]);
        project.skip_update = Some(true);
        let t = tree(project, ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(r.skip_update);
    }

    #[test]
    fn skip_update_local_false_overrides_project_true() {
        let mut project = ace("s", &[]);
        project.skip_update = Some(true);
        let mut local = ace("s", &[]);
        local.skip_update = Some(false);
        let t = tree(project, local);
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(!r.skip_update, "local false should override project true");
    }

    #[test]
    fn skip_update_user_true_used_when_others_unset() {
        let t = Tree {
            ace_user: AceToml { skip_update: Some(true), ..AceToml::default() },
            ace_project: AceToml::default(),
            ace_local: AceToml::default(),
            school_backend: None,
            school_toml: None,
            school_paths: None,
        };
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(r.skip_update);
    }

    #[test]
    fn skip_update_project_overrides_user() {
        let mut project = ace("s", &[]);
        project.skip_update = Some(false);
        let t = Tree {
            ace_user: AceToml { skip_update: Some(true), ..AceToml::default() },
            ace_project: project,
            ace_local: AceToml::default(),
            school_backend: None,
            school_toml: None,
            school_paths: None,
        };
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(!r.skip_update, "project false should override user true");
    }

    #[test]
    fn resume_defaults_true() {
        let t = tree(ace("s", &[]), ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(r.resume);
    }

    #[test]
    fn resume_local_false_disables() {
        let mut local = ace("s", &[]);
        local.resume = Some(false);
        let t = tree(ace("s", &[]), local);
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(!r.resume);
    }

    #[test]
    fn resume_project_ignored() {
        let mut project = ace("s", &[]);
        project.resume = Some(false);
        let t = tree(project, ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default());
        assert!(r.resume, "project-level resume=false should be ignored");
    }
}
