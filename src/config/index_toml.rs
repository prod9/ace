use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::paths::cache_dir;

/// ~/.cache/ace/index.toml — tracks downloaded schools.
///
/// ```toml
/// [[school]]
/// specifier = "prod9/school"
/// repo = "prod9/school"
/// path = ""
///
/// [[school]]
/// specifier = "prod9/mono:school"
/// repo = "prod9/mono"
/// path = "school"
/// ```
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct IndexToml {
    #[serde(default)]
    pub school: Vec<SchoolEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SchoolEntry {
    pub specifier: String,
    pub repo: String,
    pub path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("bad index: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("{0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("{0}")]
    Path(#[from] super::paths::PathError),
}

pub fn index_path() -> Result<PathBuf, IndexError> {
    let path = cache_dir()?.join("ace").join("index.toml");
    Ok(path)
}

pub fn load(path: &Path) -> Result<IndexToml, IndexError> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(IndexToml::default()),
        Err(e) => return Err(e.into()),
    };
    let index: IndexToml = toml::from_str(&content)?;
    Ok(index)
}

pub fn save(path: &Path, index: &IndexToml) -> Result<(), IndexError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string(index)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Parse a specifier into (repo, path) components.
fn split_specifier(specifier: &str) -> (&str, &str) {
    match specifier.split_once(':') {
        Some((repo, path)) => {
            let path = path.trim_start_matches('/');
            (repo, path)
        }
        None => (specifier, ""),
    }
}

/// Add or update a school entry in the index. Deduplicates by specifier.
pub fn upsert(index: &mut IndexToml, specifier: &str) {
    let (repo, path) = split_specifier(specifier);
    let entry = SchoolEntry {
        specifier: specifier.to_string(),
        repo: repo.to_string(),
        path: path.to_string(),
    };

    if let Some(existing) = index.school.iter_mut().find(|s| s.specifier == specifier) {
        *existing = entry;
    } else {
        index.school.push(entry);
    }
}

/// List all cached school specifiers.
pub fn list_specifiers(index: &IndexToml) -> Vec<String> {
    index.school.iter().map(|s| s.specifier.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_specifier_simple() {
        assert_eq!(split_specifier("prod9/school"), ("prod9/school", ""));
    }

    #[test]
    fn split_specifier_with_path() {
        assert_eq!(split_specifier("prod9/mono:school"), ("prod9/mono", "school"));
    }

    #[test]
    fn split_specifier_with_leading_slash() {
        assert_eq!(split_specifier("prod9/mono:/school"), ("prod9/mono", "school"));
    }

    #[test]
    fn upsert_adds_new() {
        let mut index = IndexToml::default();
        upsert(&mut index, "prod9/school");
        assert_eq!(index.school.len(), 1);
        assert_eq!(index.school[0].specifier, "prod9/school");
        assert_eq!(index.school[0].repo, "prod9/school");
        assert_eq!(index.school[0].path, "");
    }

    #[test]
    fn upsert_with_path() {
        let mut index = IndexToml::default();
        upsert(&mut index, "prod9/mono:school");
        assert_eq!(index.school[0].repo, "prod9/mono");
        assert_eq!(index.school[0].path, "school");
    }

    #[test]
    fn upsert_deduplicates() {
        let mut index = IndexToml::default();
        upsert(&mut index, "prod9/school");
        upsert(&mut index, "prod9/school");
        assert_eq!(index.school.len(), 1);
    }

    #[test]
    fn upsert_multiple_schools() {
        let mut index = IndexToml::default();
        upsert(&mut index, "prod9/school");
        upsert(&mut index, "acme/school");
        assert_eq!(index.school.len(), 2);
    }

    #[test]
    fn list_specifiers_returns_all() {
        let mut index = IndexToml::default();
        upsert(&mut index, "prod9/school");
        upsert(&mut index, "acme/school");
        let specs = list_specifiers(&index);
        assert_eq!(specs, vec!["prod9/school", "acme/school"]);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let path = Path::new("/tmp/ace-test-nonexistent/index.toml");
        let index = load(path).expect("missing file should return default");
        assert!(index.school.is_empty());
    }

    #[test]
    fn roundtrip_save_load() {
        let dir = std::env::temp_dir().join("ace-test-index-roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("index.toml");

        let mut index = IndexToml::default();
        upsert(&mut index, "prod9/school");
        upsert(&mut index, "prod9/mono:school");

        save(&path, &index).expect("save should succeed");
        let loaded = load(&path).expect("load should succeed");

        assert_eq!(loaded.school.len(), 2);
        assert_eq!(loaded.school[0].specifier, "prod9/school");
        assert_eq!(loaded.school[1].specifier, "prod9/mono:school");
        assert_eq!(loaded.school[1].repo, "prod9/mono");
        assert_eq!(loaded.school[1].path, "school");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
