use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ~/.config/ace/config.toml
///
/// Top-level keys are school identifiers ("owner/repo").
///
/// ```toml
/// ["acme-corp/school"]
///
/// ["myuser/school"]
/// ```
pub type UserConfig = HashMap<String, SchoolEntry>;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SchoolEntry {}
