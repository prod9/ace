use std::path::PathBuf;

use super::ConfigError;
use crate::paths;

pub struct AcePaths {
    pub user: PathBuf,
    pub project: PathBuf,
    pub local: PathBuf,
    pub cache: PathBuf,
}

pub fn resolve(project_dir: &std::path::Path) -> Result<AcePaths, ConfigError> {
    let user = paths::user_config_dir().ok_or(ConfigError::NoConfigDir)?.join("ace/ace.toml");
    let project = project_dir.join("ace.toml");
    let local = project_dir.join("ace.local.toml");
    let cache = ace_cache_dir()?;

    Ok(AcePaths { user, project, local, cache })
}

/// ACE's cache root: `<user_cache_dir>/ace`.
pub fn ace_cache_dir() -> Result<PathBuf, ConfigError> {
    paths::user_cache_dir()
        .ok_or(ConfigError::NoCacheDir)
        .map(|d| d.join("ace"))
}
