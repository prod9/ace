use std::path::Path;

use crate::ace::Ace;
use crate::config::school_toml::{self, ServiceDecl};
use crate::config::ConfigError;

#[derive(Debug, thiserror::Error)]
pub enum AddServiceError {
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("service already exists: {0}")]
    DuplicateService(String),
}

pub struct AddService<'a> {
    pub school_root: &'a Path,
    pub service: ServiceDecl,
}

impl AddService<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<(), AddServiceError> {
        let toml_path = self.school_root.join("school.toml");
        let mut school = school_toml::load(&toml_path)?;

        if school.services.iter().any(|s| s.name == self.service.name) {
            return Err(AddServiceError::DuplicateService(self.service.name.clone()));
        }

        school.services.push(self.service.clone());
        school_toml::save(&toml_path, &school)?;

        ace.done(&format!("Added service \"{}\"", self.service.name));
        Ok(())
    }
}
