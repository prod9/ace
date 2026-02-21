use std::path::Path;

use crate::config;
use crate::session::Session;
use crate::state::setup::SetupError;

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

        let options: Vec<&str> = schools.iter().map(|s| s.as_str()).collect();
        let idx = session.ui.select("Select school:", &options).await;
        let specifier = schools.get(idx).ok_or(SetupError::NoCachedSchools)?.clone();

        let ace_paths = config::paths::resolve(self.project_dir)?;
        WriteConfig::project(&ace_paths.project, &specifier)?;

        session.state.school_specifier = Some(specifier);

        Ok(())
    }
}

fn list_cached_schools() -> Result<Vec<String>, SetupError> {
    // TODO: scan ~/.cache/ace/ for cached school directories
    Ok(vec![])
}
