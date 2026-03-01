use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ~/.config/ace/config.toml
///
/// Top-level keys are school identifiers ("owner/repo").
/// Each school has a `services` map of service name -> credentials.
///
/// ```toml
/// ["acme-corp/school".services.github]
/// token = "gho_..."
///
/// ["acme-corp/school".services.jira]
/// token = "eyJ..."
/// username = "alice"
/// ```
pub type UserConfig = HashMap<String, SchoolCredentials>;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SchoolCredentials {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub services: HashMap<String, ServiceCredentials>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ServiceCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, String>,
}

