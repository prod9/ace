pub mod ace_toml;
pub mod paths;
pub mod school;
pub mod school_toml;
pub mod service;
pub mod user_config;

use std::collections::HashMap;

pub struct Config {
    pub school_specifier: Option<String>,
    pub schools: HashMap<String, school::School>,
}
