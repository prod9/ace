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

pub struct State {
    pub school_specifier: Option<String>,
    pub env: HashMap<String, String>,
}

impl State {
    pub fn load(project_dir: &Path) -> Result<Self, StateError> {
        let paths = config::paths::resolve(project_dir)?;
        let merged = merge_ace_configs(&paths)?;
        Ok(Self {
            school_specifier: Some(merged.school),
            env: merged.env,
        })
    }

    pub fn empty() -> Self {
        Self {
            school_specifier: None,
            env: HashMap::new(),
        }
    }
}

fn merge_ace_configs(paths: &AcePaths) -> Result<AceToml, StateError> {
    let candidates = [&paths.user, &paths.project, &paths.local];

    let mut result: Option<AceToml> = None;
    for path in candidates {
        match config::ace_toml::load(path) {
            Ok(config) => {
                result = Some(match result {
                    Some(base) => merge(base, config),
                    None => config,
                });
            }
            Err(ConfigError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        }
    }

    result.ok_or(StateError::NoConfig)
}

fn merge(mut base: AceToml, other: AceToml) -> AceToml {
    base.school = other.school;
    for (k, v) in other.env {
        base.env.insert(k, v);
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toml(school: &str, env: &[(&str, &str)]) -> AceToml {
        AceToml {
            school: school.to_string(),
            session_prompt: String::new(),
            env: env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }

    #[test]
    fn merge_school_override() {
        let base = toml("base-school", &[]);
        let over = toml("override-school", &[]);
        let merged = merge(base, over);
        assert_eq!(merged.school, "override-school");
    }

    #[test]
    fn merge_env_override() {
        let base = toml("s", &[("KEY", "old"), ("KEEP", "yes")]);
        let over = toml("s", &[("KEY", "new")]);
        let merged = merge(base, over);
        assert_eq!(merged.env["KEY"], "new");
        assert_eq!(merged.env["KEEP"], "yes");
    }

    #[test]
    fn merge_env_additive() {
        let base = toml("s", &[("A", "1")]);
        let over = toml("s", &[("B", "2")]);
        let merged = merge(base, over);
        assert_eq!(merged.env["A"], "1");
        assert_eq!(merged.env["B"], "2");
    }
}
