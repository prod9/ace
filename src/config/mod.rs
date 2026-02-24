pub mod ace_toml;
pub mod backend;
pub mod index_toml;
pub mod paths;
pub mod school_paths;
pub mod school_toml;
pub mod tree;
pub mod user_config;

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

    // school_paths
    #[error("traversal in source: {0}")]
    TraversalInSource(String),
    #[error("traversal in path: {0}")]
    TraversalInPath(String),
}
