use crate::config::school_paths::SchoolPaths;
use crate::session::Session;
use super::setup::SetupError;

pub struct CloneSchool<'a> {
    pub paths: &'a SchoolPaths,
}

impl CloneSchool<'_> {
    pub async fn run(&self, _session: &mut Session<'_>) -> Result<(), SetupError> {
        // TODO: git clone into paths.cache, or git fetch if already cached
        Ok(())
    }
}
