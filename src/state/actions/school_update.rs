use std::collections::HashMap;
use std::path::Path;

use crate::config;
use super::import_skill::{discover_skills, ImportError};

pub struct SchoolUpdate<'a> {
    pub school_root: &'a Path,
}

#[derive(Debug, thiserror::Error)]
pub enum SchoolUpdateError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] config::ConfigError),
    #[error("{0}")]
    Import(#[from] ImportError),
}

pub enum SchoolUpdateResult {
    NoImports,
    Updated { count: usize },
}

impl SchoolUpdate<'_> {
    pub fn run(&self) -> Result<SchoolUpdateResult, SchoolUpdateError> {
        let toml_path = self.school_root.join("school.toml");
        let school = config::school_toml::load(&toml_path)?;

        if school.imports.is_empty() {
            return Ok(SchoolUpdateResult::NoImports);
        }

        // Group imports by source to avoid cloning the same repo twice.
        let mut by_source: HashMap<&str, Vec<&str>> = HashMap::new();
        for imp in &school.imports {
            by_source.entry(imp.source.as_str())
                .or_default()
                .push(imp.skill.as_str());
        }

        let mut count = 0;
        for (source, skill_names) in &by_source {
            let tmp = tempfile::tempdir()?;
            clone_repo(source, tmp.path())?;

            let discovered = discover_skills(tmp.path())?;
            for name in skill_names {
                let found = discovered.iter().find(|s| s.name == *name);
                let skill = match found {
                    Some(s) => s,
                    None => {
                        eprintln!("warning: skill {name} not found in {source}, skipping");
                        continue;
                    }
                };

                let dest = self.school_root.join("skills").join(name);
                if dest.exists() {
                    std::fs::remove_dir_all(&dest)?;
                }
                copy_dir_recursive(&skill.path, &dest)?;
                count += 1;
            }
        }

        Ok(SchoolUpdateResult::Updated { count })
    }
}

fn clone_repo(source: &str, dest: &Path) -> Result<(), ImportError> {
    let url = format!("https://github.com/{source}.git");
    let status = std::process::Command::new("git")
        .args(["clone", "--depth", "1", "--single-branch", "--no-tags", &url])
        .arg(dest)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| ImportError::Clone(format!("git clone: {e}")))?;

    if !status.success() {
        return Err(ImportError::Clone(format!("git clone exited {status}")));
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
