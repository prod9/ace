use std::path::Path;

use crate::state::setup::SetupError;

pub struct WriteConfig;

impl WriteConfig {
    pub fn user(_path: &Path, _specifier: &str) -> Result<(), SetupError> {
        // TODO: create/update ~/.config/ace/config.toml with school entry
        Ok(())
    }

    pub fn project(_path: &Path, _specifier: &str) -> Result<(), SetupError> {
        // TODO: write ace.toml with school = "<specifier>"
        Ok(())
    }
}
