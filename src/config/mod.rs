pub mod school;
pub mod service;

use std::collections::HashMap;

pub struct Config {
    pub schools: HashMap<String, school::School>,
}
