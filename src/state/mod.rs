mod backend_resolve;
pub mod discover;
pub mod school;
pub mod skills;

pub use school::School;

use std::collections::HashMap;

use crate::backend::Backend;
use crate::config::ace_toml::{AceToml, Trust};
use crate::config::tree::Tree;
use crate::config::ConfigError;
use crate::resolver;

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
    pub trust: Trust,
    pub resume: bool,
    pub skip_update: bool,
}

impl State {
    /// Full resolution: resolve all effective values from config layers.
    /// Tree must have `load_school()` called first if school.toml is needed.
    pub fn resolve(tree: Tree, overrides: RuntimeOverrides) -> Result<Self, ConfigError> {
        let resolved = resolve_layers(&tree, overrides)?;
        Ok(Self {
            school_specifier: resolved.school_specifier,
            backend: resolved.backend,
            session_prompt: resolved.session_prompt,
            env: resolved.env,
            trust: resolved.trust,
            resume: resolved.resume,
            skip_update: resolved.skip_update,
            config: tree,
        })
    }

    #[allow(dead_code)] // used in tests + future use
    pub fn empty() -> Self {
        Self {
            config: Tree::default(),
            school_specifier: None,
            backend: Backend::default(),
            session_prompt: String::new(),
            env: HashMap::new(),
            trust: Trust::Default,
            resume: true,
            skip_update: false,
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

/// Resolve effective values from layers via the unified `resolver::merge`.
///
/// Pure scalar merging happens in `resolver/`. This function adds the
/// fallible binding step: build the backend registry from the merged decls
/// and look up the selected name. Unknown name → `UnknownBackend`.
fn resolve_layers(tree: &Tree, overrides: RuntimeOverrides) -> Result<Resolved, ConfigError> {
    let overrides_layer = AceToml {
        backend: overrides.backend,
        ..AceToml::default()
    };
    let merged = resolver::merge(tree, &overrides_layer);

    let registry = backend_resolve::build_registry(
        merged.backend_decls.iter().map(|s| &s.value),
    )?;
    let backend_name = merged.backend_name.value;
    let backend = registry
        .lookup(&backend_name)
        .cloned()
        .ok_or(ConfigError::UnknownBackend(backend_name))?;

    let env = merged
        .env
        .into_iter()
        .map(|(k, v)| (k, v.value))
        .collect();

    Ok(Resolved {
        school_specifier: merged.school_specifier.value,
        backend,
        session_prompt: merged.session_prompt.value,
        env,
        trust: merged.trust.value,
        resume: merged.resume.value,
        skip_update: merged.skip_update.value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Kind;

    fn ace(school: &str, env: &[(&str, &str)]) -> AceToml {
        AceToml {
            school: school.to_string(),
            env: env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            ..AceToml::default()
        }
    }

    fn tree(project: AceToml, local: AceToml) -> Tree {
        Tree {
            user: None,
            project: Some(project),
            local: Some(local),
            school: None,
        }
    }

    fn tree_with_school_backend(
        project: AceToml,
        local: AceToml,
        backend: &str,
    ) -> Tree {
        use crate::config::school_toml::SchoolToml;
        Tree {
            user: None,
            project: Some(project),
            local: Some(local),
            school: Some(SchoolToml {
                backend: Some(backend.to_string()),
                ..SchoolToml::default()
            }),
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
        let t = tree_with_school_backend(ace("", &[]), ace("", &[]), Kind::Codex.name());

        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Codex);
    }

    #[test]
    fn resolve_backend_project_overrides_school() {
        let mut project = ace("", &[]);
        project.backend = Some(Kind::Claude.into());

        let t = tree_with_school_backend(project, ace("", &[]), Kind::Codex.name());

        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Claude);
    }

    #[test]
    fn resolve_backend_local_overrides_school() {
        let mut local = ace("", &[]);
        local.backend = Some(Kind::Claude.into());

        let t = tree_with_school_backend(ace("", &[]), local, Kind::Codex.name());

        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert_eq!(r.backend.kind, Kind::Claude);
    }

    #[test]
    fn resolve_backend_override_beats_all_layers() {
        let mut project = ace("", &[]);
        project.backend = Some(Kind::Flaude.into());

        let mut local = ace("", &[]);
        local.backend = Some(Kind::Claude.into());

        let t = tree_with_school_backend(project, local, Kind::Codex.name());

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
            user: Some(AceToml { skip_update: Some(true), ..AceToml::default() }),
            project: None,
            local: None,
            school: None,
        };
        let r = resolve_layers(&t, RuntimeOverrides::default()).expect("resolve");
        assert!(r.skip_update);
    }

    #[test]
    fn skip_update_project_overrides_user() {
        let mut project = ace("s", &[]);
        project.skip_update = Some(false);
        let t = Tree {
            user: Some(AceToml { skip_update: Some(true), ..AceToml::default() }),
            project: Some(project),
            local: None,
            school: None,
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
