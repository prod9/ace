use std::path::Path;

use crate::config;
use crate::session::Session;
use super::setup::SetupError;

use super::authenticate::Authenticate;
use super::download_school::DownloadSchool;
use super::sync_skills::SyncSkills;
use super::write_config::WriteConfig;

pub struct Install<'a> {
    pub project_dir: &'a Path,
    pub specifier: &'a str,
}

impl Install<'_> {
    pub async fn run(&self, session: &mut Session<'_>) -> Result<(), SetupError> {
        let school_paths = config::school_paths::resolve(self.project_dir, self.specifier)?;

        DownloadSchool { paths: &school_paths }.run(session)?;

        let school_toml_path = school_paths.root.join("school.toml");
        let school_toml = config::school_toml::load(&school_toml_path)?;

        println!("School: {}", school_toml.school.name);

        for service in &school_toml.services {
            Authenticate { service }.run(session).await?;
        }

        let ace_paths = config::paths::resolve(self.project_dir)?;
        WriteConfig::user(&ace_paths.user, self.specifier)?;
        WriteConfig::project(&ace_paths.project, self.specifier)?;

        let result = SyncSkills {
            school_root: &school_paths.root,
            project_dir: self.project_dir,
        }
        .run(session)?;

        if result.synced > 0 {
            println!("Synced {} skills", result.synced);
        }

        session.state.school_specifier = Some(self.specifier.to_string());

        Ok(())
    }
}
