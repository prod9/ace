use std::collections::HashMap;

use super::Service;

pub struct School {
    pub source: String,
    pub services: HashMap<String, Service>,
}
