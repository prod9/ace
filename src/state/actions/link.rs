use std::path::Path;

use crate::config;
use crate::config::index_toml;
use crate::session::Session;
use super::setup::SetupError;

use super::write_config::WriteConfig;

pub struct Link<'a> {
    pub project_dir: &'a Path,
}

impl Link<'_> {
    pub async fn run(&self, session: &mut Session<'_>) -> Result<(), SetupError> {
        let schools = list_cached_schools()?;
        if schools.is_empty() {
            return Err(SetupError::NoCachedSchools);
        }

        // Convention: one school → use it. Multiple → use first (TUI selection later).
        let specifier = schools.first().ok_or(SetupError::NoCachedSchools)?.clone();

        if schools.len() > 1 {
            eprintln!("Multiple schools cached, using: {specifier}");
            eprintln!("(TUI school picker coming soon)");
        }

        let ace_paths = config::paths::resolve(self.project_dir)?;
        WriteConfig::project(&ace_paths.project, &specifier)?;

        session.state.school_specifier = Some(specifier);

        Ok(())
    }
}

fn list_cached_schools() -> Result<Vec<String>, SetupError> {
    let index_path = index_toml::index_path()
        .map_err(|e| SetupError::Clone(format!("index path: {e}")))?;
    let index = index_toml::load(&index_path)
        .map_err(|e| SetupError::Clone(format!("load index: {e}")))?;
    Ok(index_toml::list_specifiers(&index))
}
