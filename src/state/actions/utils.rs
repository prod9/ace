use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct DiscoveredSkill {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum CloneError {
    #[error("clone failed: {0}")]
    Clone(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub fn clone_repo(source: &str, dest: &Path) -> Result<(), CloneError> {
    let url = format!("https://github.com/{source}.git");
    crate::git::clone_shallow(&url, dest)
        .map_err(|e| CloneError::Clone(e.to_string()))
}

/// Discover skills by finding SKILL.md files in the repo.
/// Searches both root-level dirs and `skills/` subdirectory.
pub fn discover_skills(dir: &Path) -> Result<Vec<DiscoveredSkill>, std::io::Error> {
    let mut skills = Vec::new();
    let mut seen = HashSet::new();

    // Check `skills/` subdirectory first (preferred convention)
    let skills_dir = dir.join("skills");
    if skills_dir.is_dir() {
        scan_for_skills(&skills_dir, &mut skills, &mut seen)?;
    }

    // Also check root-level directories
    scan_for_skills(dir, &mut skills, &mut seen)?;

    Ok(skills)
}

fn scan_for_skills(
    parent: &Path,
    skills: &mut Vec<DiscoveredSkill>,
    seen: &mut HashSet<String>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if name.starts_with('.') {
            continue;
        }

        if path.join("SKILL.md").exists() && seen.insert(name.clone()) {
            skills.push(DiscoveredSkill { name, path });
        }
    }
    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
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
