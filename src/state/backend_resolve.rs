//! Merge `[[backends]]` declarations into a `Backend` registry.
//!
//! Layer-walk logic lives here so `Registry` (in `src/backend/`) stays
//! independent of config-layer types.

use std::path::Path;

use crate::backend::{Backend, Kind, Registry};
use crate::config::ace_toml::BackendDecl;
use crate::config::ConfigError;

/// Build a `Registry` seeded with built-ins, then fold each declaration in
/// order. Caller controls layer order (typically school → user → project →
/// local). Per-decl rule documented on `merge_decl`.
pub fn build_registry<'a, I>(decls: I) -> Result<Registry, ConfigError>
where
    I: IntoIterator<Item = &'a BackendDecl>,
{
    let mut registry = Registry::with_builtins();
    for decl in decls {
        merge_decl(&mut registry, decl)?;
    }
    Ok(registry)
}

/// Merge a single `BackendDecl` into the registry.
///
/// Rule:
/// - If `decl.name` already registered (built-in or earlier-layer custom):
///   partial override — `env` per-key last-wins, `cmd` last-wins-non-empty,
///   `kind` (if specified) must match existing.
/// - Else (new name): resolve kind via explicit field → name match →
///   `cmd[0]` basename match → error. Resolve cmd via explicit `cmd` else
///   `[kind.name()]`. Insert.
fn merge_decl(registry: &mut Registry, decl: &BackendDecl) -> Result<(), ConfigError> {
    if let Some(existing) = registry.get_mut(&decl.name) {
        if let Some(declared) = &decl.kind
            && Kind::from_name(declared) != Some(existing.kind)
        {
            return Err(ConfigError::BackendKindMismatch {
                name: decl.name.clone(),
                declared: declared.clone(),
                actual: existing.kind.name().to_string(),
            });
        }
        if !decl.cmd.is_empty() {
            existing.cmd = decl.cmd.clone();
        }
        for (k, v) in &decl.env {
            existing.env.insert(k.clone(), v.clone());
        }
        return Ok(());
    }

    let kind = resolve_kind(decl)?;
    let cmd = if decl.cmd.is_empty() {
        vec![kind.name().to_string()]
    } else {
        decl.cmd.clone()
    };
    registry.insert(Backend {
        name: decl.name.clone(),
        kind,
        cmd,
        env: decl.env.clone(),
    });
    Ok(())
}

fn resolve_kind(decl: &BackendDecl) -> Result<Kind, ConfigError> {
    if let Some(declared) = &decl.kind {
        return Kind::from_name(declared)
            .ok_or_else(|| ConfigError::UnresolvableBackendKind(decl.name.clone()));
    }
    if let Some(k) = Kind::from_name(&decl.name) {
        return Ok(k);
    }
    if let Some(prog) = decl.cmd.first()
        && let Some(basename) = Path::new(prog).file_name().and_then(|s| s.to_str())
        && let Some(k) = Kind::from_name(basename)
    {
        return Ok(k);
    }
    Err(ConfigError::UnresolvableBackendKind(decl.name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn decl(name: &str) -> BackendDecl {
        BackendDecl {
            name: name.to_string(),
            kind: None,
            cmd: Vec::new(),
            env: HashMap::new(),
        }
    }

    #[test]
    fn env_override_on_builtin_last_wins_per_key() {
        let mut reg = Registry::with_builtins();
        let mut d = decl("claude");
        d.env.insert("A".into(), "1".into());
        d.env.insert("B".into(), "2".into());
        merge_decl(&mut reg, &d).expect("first merge");

        let mut d2 = decl("claude");
        d2.env.insert("B".into(), "two".into());
        d2.env.insert("C".into(), "3".into());
        merge_decl(&mut reg, &d2).expect("second merge");

        let claude = reg.lookup("claude").unwrap();
        assert_eq!(claude.env.get("A").map(String::as_str), Some("1"));
        assert_eq!(claude.env.get("B").map(String::as_str), Some("two"));
        assert_eq!(claude.env.get("C").map(String::as_str), Some("3"));
    }

    #[test]
    fn cmd_override_on_builtin_last_wins_nonempty() {
        let mut reg = Registry::with_builtins();
        let mut d = decl("claude");
        d.cmd = vec!["claude-bedrock".into()];
        merge_decl(&mut reg, &d).expect("merge");

        assert_eq!(reg.lookup("claude").unwrap().cmd, vec!["claude-bedrock"]);

        let d2 = decl("claude"); // empty cmd — must not clobber
        merge_decl(&mut reg, &d2).expect("merge2");
        assert_eq!(reg.lookup("claude").unwrap().cmd, vec!["claude-bedrock"]);
    }

    #[test]
    fn kind_mismatch_on_builtin_errors() {
        let mut reg = Registry::with_builtins();
        let mut d = decl("claude");
        d.kind = Some("codex".into());
        let err = merge_decl(&mut reg, &d).expect_err("should reject");
        match err {
            ConfigError::BackendKindMismatch { name, declared, actual } => {
                assert_eq!(name, "claude");
                assert_eq!(declared, "codex");
                assert_eq!(actual, "claude");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn new_name_explicit_kind() {
        let mut reg = Registry::with_builtins();
        let mut d = decl("bailer");
        d.kind = Some("claude".into());
        d.env.insert("ANTHROPIC_BASE_URL".into(), "https://x".into());
        merge_decl(&mut reg, &d).expect("merge");

        let bailer = reg.lookup("bailer").expect("bailer registered");
        assert_eq!(bailer.kind, Kind::Claude);
        assert_eq!(bailer.cmd, vec!["claude"]); // defaulted from kind
        assert_eq!(bailer.env.get("ANTHROPIC_BASE_URL").map(String::as_str), Some("https://x"));
    }

    #[test]
    fn new_name_inferred_from_cmd_basename() {
        let mut reg = Registry::with_builtins();
        let mut d = decl("bedrock-claude");
        d.cmd = vec!["/usr/local/bin/claude".into()];
        merge_decl(&mut reg, &d).expect("merge");

        let b = reg.lookup("bedrock-claude").unwrap();
        assert_eq!(b.kind, Kind::Claude);
        assert_eq!(b.cmd, vec!["/usr/local/bin/claude"]);
    }

    #[test]
    fn new_name_unresolvable_errors() {
        let mut reg = Registry::with_builtins();
        let d = decl("mystery"); // no kind, no cmd, name doesn't match built-in
        let err = merge_decl(&mut reg, &d).expect_err("should error");
        assert!(matches!(err, ConfigError::UnresolvableBackendKind(name) if name == "mystery"));
    }

    #[test]
    fn new_name_explicit_kind_unknown_errors() {
        let mut reg = Registry::with_builtins();
        let mut d = decl("bailer");
        d.kind = Some("nonsense".into());
        let err = merge_decl(&mut reg, &d).expect_err("should error");
        assert!(matches!(err, ConfigError::UnresolvableBackendKind(name) if name == "bailer"));
    }
}
