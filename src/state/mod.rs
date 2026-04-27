mod backend_resolve;
pub mod discover;
mod resolver;
pub mod school;
pub mod skills;

pub use school::School;

use std::collections::HashMap;

use crate::config::ace_toml::{AceToml, Trust};
use crate::backend::{Backend, Kind, Registry};
use crate::config::tree::Tree;
use crate::config::ConfigError;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuntimeOverrides {
    pub backend: Option<String>,
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
    pub fn resolve(tree: Tree, overrides: RuntimeOverrides) -> Result<Self, ConfigError> {
        let resolved = resolve_layers(&tree, overrides)?;
        let school = tree.school_toml.as_ref().map(|st| School::from(st.clone()));
        Ok(Self {
            school_specifier: resolved.school_specifier,
            backend: resolved.backend,
            session_prompt: resolved.session_prompt,
            env: resolved.env,
            trust: resolved.trust,
            resume: resolved.resume,
            skip_update: resolved.skip_update,
            school,
            config: tree,
        })
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
            backend: Kind::default().default_backend(),
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

#[derive(Debug)]
struct Resolved {
    school_specifier: Option<String>,
    backend: Backend,
    session_prompt: String,
    env: HashMap<String, String>,
    trust: Trust,
    resume: bool,
    skip_update: bool,
}

/// Resolve effective values from layers.
///
/// Backend resolution: build registry from built-ins → school → user → project → local.
/// Each `[[backends]]` decl merges via `backend_resolve::merge_decl`. The selected
/// backend name is picked by precedence (override → local → project → user → school
/// → `"claude"`) and looked up in the registry; unknown name → `UnknownBackend`.
fn resolve_layers(tree: &Tree, overrides: RuntimeOverrides) -> Result<Resolved, ConfigError> {
    let layers = [&tree.ace_user, &tree.ace_project, &tree.ace_local];

    // school: last non-empty wins
    let school_specifier = layers
        .iter()
        .rev()
        .find(|l| !l.school.is_empty())
        .map(|l| l.school.clone());

    // Build registry: built-ins → school decls → user/project/local decls.
    let mut registry = Registry::with_builtins();
    if let Some(school_toml) = &tree.school_toml {
        for decl in &school_toml.backends {
            backend_resolve::merge_decl(&mut registry, decl)?;
        }
    }
    for layer in &layers {
        for decl in &layer.backends {
            backend_resolve::merge_decl(&mut registry, decl)?;
        }
    }

    // Selected backend name: override → local → project → user → school → "claude"
    let backend_name = overrides.backend
        .or_else(|| tree.ace_local.backend.clone())
        .or_else(|| tree.ace_project.backend.clone())
        .or_else(|| tree.ace_user.backend.clone())
        .or_else(|| tree.school_backend.clone())
        .unwrap_or_else(|| Kind::default().into());

    let backend = registry
        .lookup(&backend_name)
        .cloned()
        .ok_or(ConfigError::UnknownBackend(backend_name))?;

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

    Ok(Resolved {
        school_specifier,
        backend,
        session_prompt,
        env,
        trust,
        resume,
        skip_update,
    })
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
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.school_specifier.as_deref(), Some("project-school"));
    }

    #[test]
    fn resolve_school_local_overrides() {
        let t = tree(
            ace("project-school", &[]),
            ace("local-school", &[]),
        );
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.school_specifier.as_deref(), Some("local-school"));
    }

    #[test]
    fn resolve_school_none_when_all_empty() {
        let t = tree(ace("", &[]), ace("", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert!(r.school_specifier.is_none());
    }

    #[test]
    fn resolve_backend_local_overrides_project() {
        let mut project = ace("", &[]);
        project.backend = Some(Kind::Codex.into());
        let mut local = ace("", &[]);
        local.backend = Some(Kind::Claude.into());

        let t = tree(project, local);
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Claude);
    }

    #[test]
    fn resolve_backend_fallback_claude() {
        let t = tree(ace("", &[]), ace("", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Claude);
    }

    #[test]
    fn resolve_env_additive() {
        let t = tree(
            ace("s", &[("A", "1")]),
            ace("s", &[("B", "2")]),
        );
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.env["A"], "1");
        assert_eq!(r.env["B"], "2");
    }

    #[test]
    fn resolve_env_override() {
        let t = tree(
            ace("s", &[("KEY", "old"), ("KEEP", "yes")]),
            ace("s", &[("KEY", "new")]),
        );
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.env["KEY"], "new");
        assert_eq!(r.env["KEEP"], "yes");
    }

    #[test]
    fn resolve_session_prompt_last_wins() {
        let mut project = ace("", &[]);
        project.session_prompt = Some("project prompt".to_string());

        let t = tree(project, ace("", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.session_prompt, "project prompt");
    }

    #[test]
    fn resolve_backend_school_toml_used() {
        let mut t = tree(ace("", &[]), ace("", &[]));
        t.school_backend = Some(Kind::Codex.into());

        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Codex);
    }

    #[test]
    fn resolve_backend_project_overrides_school() {
        let mut project = ace("", &[]);
        project.backend = Some(Kind::Claude.into());

        let mut t = tree(project, ace("", &[]));
        t.school_backend = Some(Kind::Codex.into());

        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Claude);
    }

    #[test]
    fn resolve_backend_local_overrides_school() {
        let mut local = ace("", &[]);
        local.backend = Some(Kind::Claude.into());

        let mut t = tree(ace("", &[]), local);
        t.school_backend = Some(Kind::Codex.into());

        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Claude);
    }

    #[test]
    fn resolve_backend_override_beats_all_layers() {
        let mut project = ace("", &[]);
        project.backend = Some(Kind::Flaude.into());

        let mut local = ace("", &[]);
        local.backend = Some(Kind::Claude.into());

        let mut t = tree(project, local);
        t.school_backend = Some(Kind::Codex.into());

        let r = resolve_layers(
            &t,
            RuntimeOverrides {
                backend: Some(Kind::Codex.into()),
            },
        ).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Codex);
    }

    #[test]
    fn resolve_per_backend_env_merges_into_backend() {
        use crate::config::ace_toml::BackendDecl;

        let mut project = ace("s", &[]);
        project.backend = Some(Kind::Claude.into());
        project.backends = vec![BackendDecl {
            name: "claude".into(),
            kind: None,
            cmd: Vec::new(),
            env: [("API_BASE".to_string(), "https://example.com".to_string())]
                .into_iter()
                .collect(),
        }];

        let t = tree(project, ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");

        assert_eq!(r.backend.kind, Kind::Claude);
        assert_eq!(r.backend.name, "claude");
        assert_eq!(r.backend.env.get("API_BASE").map(String::as_str), Some("https://example.com"));
    }

    #[test]
    fn resolve_custom_backend_selectable_by_name() {
        use crate::config::ace_toml::BackendDecl;

        let mut project = ace("s", &[]);
        project.backend = Some("bailer".into());
        project.backends = vec![BackendDecl {
            name: "bailer".into(),
            kind: Some(Kind::Claude.into()),
            cmd: Vec::new(),
            env: [("ANTHROPIC_BASE_URL".to_string(), "https://x".to_string())]
                .into_iter().collect(),
        }];

        let t = tree(project, ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");

        assert_eq!(r.backend.name, "bailer");
        assert_eq!(r.backend.kind, Kind::Claude);
        assert_eq!(r.backend.cmd, vec!["claude"]);
        assert_eq!(r.backend.env.get("ANTHROPIC_BASE_URL").map(String::as_str), Some("https://x"));
    }

    #[test]
    fn resolve_unknown_backend_name_errors() {
        let mut project = ace("s", &[]);
        project.backend = Some("nonsense".into());
        let t = tree(project, ace("s", &[]));
        let err = resolve_layers(&t, RuntimeOverrides::default()).expect_err("should error");
        assert!(matches!(err, ConfigError::UnknownBackend(name) if name == "nonsense"));
    }

    #[test]
    fn resolve_per_backend_env_layer_collision_local_wins() {
        use crate::config::ace_toml::BackendDecl;

        let mut project = ace("s", &[]);
        project.backend = Some(Kind::Claude.into());
        project.backends = vec![BackendDecl {
            name: "claude".into(),
            kind: None,
            cmd: Vec::new(),
            env: [
                ("KEEP".to_string(), "yes".to_string()),
                ("KEY".to_string(), "old".to_string()),
            ].into_iter().collect(),
        }];

        let mut local = ace("s", &[]);
        local.backends = vec![BackendDecl {
            name: "claude".into(),
            kind: None,
            cmd: Vec::new(),
            env: [("KEY".to_string(), "new".to_string())].into_iter().collect(),
        }];

        let t = tree(project, local);
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");

        assert_eq!(r.backend.env.get("KEY").map(String::as_str), Some("new"));
        assert_eq!(r.backend.env.get("KEEP").map(String::as_str), Some("yes"));
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
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert!(!r.skip_update);
    }

    #[test]
    fn skip_update_project_true() {
        let mut project = ace("s", &[]);
        project.skip_update = Some(true);
        let t = tree(project, ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert!(r.skip_update);
    }

    #[test]
    fn skip_update_local_false_overrides_project_true() {
        let mut project = ace("s", &[]);
        project.skip_update = Some(true);
        let mut local = ace("s", &[]);
        local.skip_update = Some(false);
        let t = tree(project, local);
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
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
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
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
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert!(!r.skip_update, "project false should override user true");
    }

    #[test]
    fn resume_defaults_true() {
        let t = tree(ace("s", &[]), ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert!(r.resume);
    }

    #[test]
    fn resume_local_false_disables() {
        let mut local = ace("s", &[]);
        local.resume = Some(false);
        let t = tree(ace("s", &[]), local);
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert!(!r.resume);
    }

    #[test]
    fn resume_project_ignored() {
        let mut project = ace("s", &[]);
        project.resume = Some(false);
        let t = tree(project, ace("s", &[]));
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert!(r.resume, "project-level resume=false should be ignored");
    }
}
