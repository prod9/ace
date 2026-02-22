use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("neither XDG_CONFIG_HOME nor HOME is set")]
    NoConfigDir,
    #[error("neither XDG_CACHE_HOME nor HOME is set")]
    NoCacheDir,
}

pub struct AcePaths {
    pub user: PathBuf,
    pub local: PathBuf,
    pub project: PathBuf,
}

pub fn resolve(project_dir: &std::path::Path) -> Result<AcePaths, PathError> {
    let user = config_dir()?.join("ace").join("config.toml");
    let local = project_dir.join("ace.local.toml");
    let project = project_dir.join("ace.toml");

    Ok(AcePaths { user, local, project })
}

pub(super) fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub(super) fn config_dir() -> Result<PathBuf, PathError> {
    let xdg = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let home = home_dir().map(|h| h.join(".config"));
    xdg.or_else(|| home).ok_or(PathError::NoConfigDir)
}

pub(super) fn cache_dir() -> Result<PathBuf, PathError> {
    let xdg = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from);
    let home = home_dir().map(|h| h.join(".cache"));
    xdg.or_else(|| home).ok_or(PathError::NoCacheDir)
}