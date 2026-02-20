pub mod ace_toml;
pub mod paths;
pub mod school;
pub mod school_toml;
pub mod service;
pub mod user_config;

use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
}

pub struct Config {
    pub school_specifier: Option<String>,
    pub schools: HashMap<String, school::School>,
}
