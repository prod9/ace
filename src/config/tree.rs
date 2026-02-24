use std::path::Path;

use super::ace_toml::{self, AceToml};
use super::backend::Backend;
use super::paths::AcePaths;
use super::ConfigError;

/// Raw config layers, preserved for write-back and inspection.
pub struct Tree {
    pub user: AceToml,
    pub project: AceToml,
    pub local: AceToml,
    /// Backend from school.toml, applied after load when school is known.
    pub school_backend: Option<Backend>,
}

impl Tree {
    pub fn load(paths: &AcePaths) -> Result<Self, ConfigError> {
        let user = load_or_default(&paths.user)?;
        let project = load_or_default(&paths.project)?;
        let local = load_or_default(&paths.local)?;

        let any_found = [&paths.user, &paths.project, &paths.local]
            .iter()
            .any(|p| p.exists());
        if !any_found {
            return Err(ConfigError::NoConfig);
        }

        Ok(Tree { user, project, local, school_backend: None })
    }
}

fn load_or_default(path: &Path) -> Result<AceToml, ConfigError> {
    match ace_toml::load(path) {
        Ok(config) => Ok(config),
        Err(ConfigError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(AceToml::default())
        }
        Err(e) => Err(e),
    }
}
