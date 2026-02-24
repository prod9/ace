use std::path::Path;
use std::process::Command;

use crate::config;
use crate::config::index_toml;
use crate::session::Session;
use super::prepare::PrepareError;

use super::authenticate::Authenticate;
use super::write_config::WriteConfig;

/// First-time school setup: git clone + auth + write user config.
pub struct Install<'a> {
    pub project_dir: &'a Path,
    pub specifier: &'a str,
}

impl Install<'_> {
    pub async fn run(&self, session: &mut Session<'_>) -> Result<(), PrepareError> {
        let school_paths = config::school_paths::resolve(self.project_dir, self.specifier)?;
        let cache = match &school_paths.cache {
            Some(c) => c,
            None => return Ok(()), // embedded school
        };

        if let Some(parent) = cache.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| PrepareError::Clone(format!("mkdir: {e}")))?;
        }
        let repo = self.specifier.split_once(':').map_or(
            self.specifier,
            |(owner_repo, _)| owner_repo,
        );
        let url = format!("https://github.com/{repo}.git");

        session.progress(&format!("Cloning {repo}"));
        let status = Command::new("git")
            .args(["clone", "--depth", "1", "--single-branch", "--no-tags", &url])
            .arg(cache)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| PrepareError::Clone(format!("git clone: {e}")))?;
        if !status.success() {
            return Err(PrepareError::Clone(format!("git clone exited {status}")));
        }
        session.done(&format!("Cloned {repo}"));

        update_index(&school_paths.source)?;
        let school_toml_path = school_paths.root.join("school.toml");
        let school_toml = config::school_toml::load(&school_toml_path)?;
        println!("School: {}", school_toml.school.name);

        for service in &school_toml.services {
            Authenticate { service }.run(session).await?;
        }

        let ace_paths = config::paths::resolve(self.project_dir)?;
        WriteConfig::user(&ace_paths.user, self.specifier)?;

        Ok(())
    }
}

fn update_index(source: &str) -> Result<(), PrepareError> {
    let index_path = index_toml::index_path()
        .map_err(|e| PrepareError::Clone(format!("index path: {e}")))?;
    let mut index = index_toml::load(&index_path)
        .map_err(|e| PrepareError::Clone(format!("load index: {e}")))?;
    index_toml::upsert(&mut index, source);
    index_toml::save(&index_path, &index)
        .map_err(|e| PrepareError::Clone(format!("save index: {e}")))?;
    Ok(())
}
