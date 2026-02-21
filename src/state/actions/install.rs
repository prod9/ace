use std::path::Path;

use crate::config;
use crate::session::Session;
use super::setup::SetupError;

use super::authenticate::Authenticate;
use super::clone_school::CloneSchool;
use super::write_config::WriteConfig;

pub struct Install<'a> {
    pub project_dir: &'a Path,
    pub specifier: &'a str,
}

impl Install<'_> {
    pub async fn run(&self, session: &mut Session<'_>) -> Result<(), SetupError> {
        let school_paths = config::school_paths::resolve(self.project_dir, self.specifier)?;

        CloneSchool { paths: &school_paths }.run(session).await?;

        let school_toml_path = school_paths.root.join("school.toml");
        let school_toml = config::school_toml::load(&school_toml_path)?;

        println!("School: {}", school_toml.school.name);

        for service in &school_toml.services {
            Authenticate { service }.run(session).await?;
        }

        let ace_paths = config::paths::resolve(self.project_dir)?;
        WriteConfig::user(&ace_paths.user, self.specifier)?;
        WriteConfig::project(&ace_paths.project, self.specifier)?;

        session.state.school_specifier = Some(self.specifier.to_string());

        Ok(())
    }
}
