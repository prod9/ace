use std::path::PathBuf;

use super::ConfigError;

pub struct AcePaths {
    pub user: PathBuf,
    pub local: PathBuf,
    pub project: PathBuf,
    pub cache: PathBuf,
}

pub fn resolve(project_dir: &std::path::Path) -> Result<AcePaths, ConfigError> {
    let user = config_dir()?.join("ace").join("config.toml");
    let local = project_dir.join("ace.local.toml");
    let project = project_dir.join("ace.toml");
    let cache = cache_dir()?.join("ace");

    Ok(AcePaths { user, local, project, cache })
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
