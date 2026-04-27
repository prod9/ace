use std::path::Path;

use super::ace_toml::{self, AceToml};
use super::paths::AcePaths;
use super::school_paths::{self, SchoolPaths};
use super::school_toml::{self, SchoolToml};
use super::ConfigError;

/// Raw config layers, preserved for write-back and inspection.
#[derive(Clone)]
pub struct Tree {
    pub ace_user: AceToml,
    pub ace_project: AceToml,
    pub ace_local: AceToml,
    /// Backend name from school.toml, applied after load when school is known.
    pub school_backend: Option<String>,
    pub school_toml: Option<SchoolToml>,
    pub school_paths: Option<SchoolPaths>,
}

impl Tree {
    pub fn load(paths: &AcePaths) -> Result<Self, ConfigError> {
        let ace_user = load_or_default(&paths.user)?;
        let ace_project = load_or_default(&paths.project)?;
        let ace_local = load_or_default(&paths.local)?;

        // User layer alone doesn't mean a project is set up.
        let any_found = [&paths.project, &paths.local]
            .iter()
            .any(|p| p.exists());
        if !any_found {
            return Err(ConfigError::NoConfig);
        }

        Ok(Tree {
            ace_user,
            ace_project,
            ace_local,
            school_backend: None,
            school_toml: None,
            school_paths: None,
        })
    }

    /// Resolve school specifier from ace.toml layers (last non-empty wins).
    pub fn specifier(&self) -> Option<String> {
        [&self.ace_local, &self.ace_project, &self.ace_user]
            .iter()
            .find(|l| !l.school.is_empty())
            .map(|l| l.school.clone())
    }

    /// Second pass: load school.toml + school_paths from the resolved specifier.
    pub fn load_school(&mut self, project_dir: &Path) -> Result<(), ConfigError> {
        let Some(spec) = self.specifier() else {
            return Ok(());
        };

        let sp = school_paths::resolve(project_dir, &spec)?;
        let school_toml_path = sp.root.join("school.toml");
        if school_toml_path.exists()
            && let Ok(st) = school_toml::load(&school_toml_path)
        {
            self.school_backend = st.backend.clone();
            self.school_toml = Some(st);
        }
        self.school_paths = Some(sp);
        Ok(())
    }
}

fn load_or_default(path: &Path) -> Result<AceToml, ConfigError> {
    match ace_toml::load(path) {
        Ok(config) => Ok(config),
        Err(ConfigError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(AceToml::default())
        }
        Err(e) => Err(e),
    }
}

