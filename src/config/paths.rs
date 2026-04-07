use std::path::PathBuf;

use super::ConfigError;

pub struct AcePaths {
    pub user: PathBuf,
    pub project: PathBuf,
    pub local: PathBuf,
    pub cache: PathBuf,
}

pub fn resolve(project_dir: &std::path::Path) -> Result<AcePaths, ConfigError> {
    let user = config_dir()?.join("ace/ace.toml");
    let project = project_dir.join("ace.toml");
    let local = project_dir.join("ace.local.toml");
    let cache = cache_dir()?.join("ace");

    Ok(AcePaths { user, project, local, cache })
}

pub(super) fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub(super) fn config_dir() -> Result<PathBuf, ConfigError> {
    let xdg = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let home = home_dir().map(|h| h.join(".config"));
    xdg.or_else(|| home).ok_or(ConfigError::NoConfigDir)
}

pub(super) fn cache_dir() -> Result<PathBuf, ConfigError> {
    let xdg = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from);
    let home = home_dir().map(|h| h.join(".cache"));
    xdg.or_else(|| home).ok_or(ConfigError::NoCacheDir)
}
