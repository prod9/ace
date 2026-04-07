pub mod ace_toml;
pub mod backend;
pub mod index_toml;
pub mod paths;
pub mod school_paths;
pub mod school_toml;
pub mod skill_meta;
pub mod tree;

use std::collections::HashMap;
use std::path::Path;

pub(crate) fn is_empty_str(s: &str) -> bool { s.is_empty() }
pub(crate) fn is_empty_map(m: &HashMap<String, String>) -> bool { m.is_empty() }
pub(crate) fn is_empty_vec<T>(v: &[T]) -> bool { v.is_empty() }

/// Config scope — determines which layer a write targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    User,
    Project,
    Local,
}

impl Scope {
    /// Default scope when no explicit flag is given, inferred from the key.
    /// Personal-only fields → Local, shared fields → Project.
    pub fn default_for_key(key: &str) -> Self {
        match key {
            "trust" | "resume" => Scope::Local,
            _ => Scope::Project,
        }
    }

    /// Resolve the filesystem path for this scope.
    pub fn path_in<'a>(&self, paths: &'a paths::AcePaths) -> &'a Path {
        match self {
            Scope::User => &paths.user,
            Scope::Project => &paths.project,
            Scope::Local => &paths.local,
        }
    }
}

/// Parsed config key for get/set operations.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigKey {
    School,
    Backend,
    Trust,
    Resume,
    SessionPrompt,
    Env(String),
}

impl ConfigKey {
    pub fn parse(key: &str) -> Option<Self> {
        if let Some(env_key) = key.strip_prefix("env.") {
            if env_key.is_empty() { return None; }
            return Some(ConfigKey::Env(env_key.to_string()));
        }

        match key {
            "school" => Some(ConfigKey::School),
            "backend" => Some(ConfigKey::Backend),
            "trust" => Some(ConfigKey::Trust),
            "resume" => Some(ConfigKey::Resume),
            "session_prompt" => Some(ConfigKey::SessionPrompt),
            _ => None,
        }
    }

    pub fn scope_key(&self) -> &str {
        match self {
            ConfigKey::School => "school",
            ConfigKey::Backend => "backend",
            ConfigKey::Trust => "trust",
            ConfigKey::Resume => "resume",
            ConfigKey::SessionPrompt => "session_prompt",
            ConfigKey::Env(_) => "env",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("bad config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("bad config: {0}")]
    Encode(#[from] toml::ser::Error),

    // paths
    #[error("neither XDG_CONFIG_HOME nor HOME is set")]
    NoConfigDir,
    #[error("neither XDG_CACHE_HOME nor HOME is set")]
    NoCacheDir,

    // tree
    #[error("no config found, ace setup?")]
    NoConfig,

    // school
    #[error("no school configured, run `ace setup`")]
    NoSchool,

    // school_paths
    #[error("traversal in source: {0}")]
    TraversalInSource(String),
    #[error("traversal in path: {0}")]
    TraversalInPath(String),
}
