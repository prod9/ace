pub mod ace_toml;
pub mod paths;
pub mod school_paths;
pub mod school_toml;
pub mod user_config;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("bad config: {0}")]
    Parse(#[from] toml::de::Error),
}
