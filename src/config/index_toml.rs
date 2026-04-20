use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::paths::cache_dir;
use super::ConfigError;

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
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct IndexToml {
    pub school: Vec<SchoolEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SchoolEntry {
    pub specifier: String,
    pub repo: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub path: String,
}

pub fn index_path() -> Result<PathBuf, ConfigError> {
    let path = cache_dir()?.join("ace").join("index.toml");
    Ok(path)
}

pub fn load(path: &Path) -> Result<IndexToml, ConfigError> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(IndexToml::default()),
        Err(e) => return Err(e.into()),
    };
    let index: IndexToml = toml::from_str(&content)?;
    Ok(index)
}

pub fn save(path: &Path, index: &IndexToml) -> Result<(), ConfigError> {
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
    fn load_missing_file_returns_default() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("missing").join("index.toml");
        let index = load(&path).expect("missing file should return default");
        assert!(index.school.is_empty());
    }

    #[test]
    fn roundtrip_save_load() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("index.toml");

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
    }
}
