use std::collections::HashMap;

pub struct Service {
    pub token: Option<String>,
    pub extra: HashMap<String, String>,
}
