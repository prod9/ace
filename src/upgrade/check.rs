use std::path::Path;
use std::time::{Duration, SystemTime};

#[allow(dead_code)] // used by startup version check (PROD9-60 task 6)
const CACHE_TTL: Duration = Duration::from_secs(4 * 3600);

pub fn parse_version_tags(tags: &[String]) -> Vec<semver::Version> {
    tags.iter()
        .filter_map(|tag| {
            let version_str = tag.strip_prefix('v')?;
            semver::Version::parse(version_str).ok()
        })
        .collect()
}

pub fn latest_version(versions: &[semver::Version]) -> Option<&semver::Version> {
    versions.iter().max()
}

#[allow(dead_code)] // used by startup version check (PROD9-60 task 6)
pub fn read_cache_marker(path: &Path) -> Option<semver::Version> {
    let content = std::fs::read_to_string(path).ok()?;
    semver::Version::parse(content.trim()).ok()
}

pub fn write_cache_marker(path: &Path, version: &semver::Version) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, version.to_string())
}

#[allow(dead_code)] // used by startup version check (PROD9-60 task 6)
pub fn is_cache_fresh(path: &Path, now: SystemTime) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let Ok(elapsed) = now.duration_since(modified) else {
        return false;
    };
    elapsed < CACHE_TTL
}

pub fn needs_update(current: &semver::Version, latest: &semver::Version) -> bool {
    latest > current
}

pub fn cache_marker_path() -> Option<std::path::PathBuf> {
    dirs::cache_dir().map(|d| d.join("ace/latest_version"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- tag parsing --

    #[test]
    fn parse_version_tags_extracts_versions() {
        let tags = vec![
            "v0.1.0".to_string(),
            "v0.2.0".to_string(),
            "v0.3.1".to_string(),
        ];
        let versions = parse_version_tags(&tags);
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0], semver::Version::new(0, 1, 0));
        assert_eq!(versions[2], semver::Version::new(0, 3, 1));
    }

    #[test]
    fn parse_version_tags_ignores_malformed() {
        let tags = vec![
            "v0.1.0".to_string(),
            "not-semver".to_string(),
            "v0.2.0".to_string(),
        ];
        let versions = parse_version_tags(&tags);
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn parse_version_tags_strips_v_prefix() {
        let tags = vec!["v1.2.3".to_string()];
        let versions = parse_version_tags(&tags);
        assert_eq!(versions[0], semver::Version::new(1, 2, 3));
    }

    #[test]
    fn parse_version_tags_empty() {
        let versions = parse_version_tags(&[]);
        assert!(versions.is_empty());
    }

    #[test]
    fn parse_version_tags_requires_v_prefix() {
        let tags = vec!["1.2.3".to_string()];
        let versions = parse_version_tags(&tags);
        assert!(versions.is_empty(), "tags without v prefix should be skipped");
    }

    // -- latest version selection --

    #[test]
    fn latest_from_tags_picks_highest() {
        let versions = vec![
            semver::Version::new(0, 1, 0),
            semver::Version::new(0, 3, 0),
            semver::Version::new(0, 2, 5),
        ];
        assert_eq!(latest_version(&versions), Some(&semver::Version::new(0, 3, 0)));
    }

    #[test]
    fn latest_from_empty_is_none() {
        let versions: Vec<semver::Version> = vec![];
        assert_eq!(latest_version(&versions), None);
    }

    // -- cache marker --

    #[test]
    fn read_cache_marker_missing_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("latest_version");
        assert!(read_cache_marker(&path).is_none());
    }

    #[test]
    fn read_cache_marker_valid() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("latest_version");
        std::fs::write(&path, "0.4.0\n").expect("write marker");
        assert_eq!(read_cache_marker(&path), Some(semver::Version::new(0, 4, 0)));
    }

    #[test]
    fn read_cache_marker_strips_whitespace() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("latest_version");
        std::fs::write(&path, "  0.4.0  \n").expect("write marker");
        assert_eq!(read_cache_marker(&path), Some(semver::Version::new(0, 4, 0)));
    }

    #[test]
    fn read_cache_marker_invalid_content() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("latest_version");
        std::fs::write(&path, "not-a-version").expect("write marker");
        assert!(read_cache_marker(&path).is_none());
    }

    #[test]
    fn write_cache_marker_creates_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("latest_version");
        let version = semver::Version::new(0, 4, 0);
        write_cache_marker(&path, &version).expect("write marker");
        let content = std::fs::read_to_string(&path).expect("read marker");
        assert_eq!(content.trim(), "0.4.0");
    }

    #[test]
    fn write_cache_marker_creates_parent_dirs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("sub/dir/latest_version");
        let version = semver::Version::new(0, 4, 0);
        write_cache_marker(&path, &version).expect("write marker");
        assert!(path.exists());
    }

    // -- cache freshness --

    #[test]
    fn cache_marker_stale_after_ttl() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("latest_version");
        std::fs::write(&path, "0.4.0").expect("write marker");

        let five_hours_later = SystemTime::now() + Duration::from_secs(5 * 3600);
        assert!(!is_cache_fresh(&path, five_hours_later));
    }

    #[test]
    fn cache_marker_fresh_within_ttl() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("latest_version");
        std::fs::write(&path, "0.4.0").expect("write marker");
        assert!(is_cache_fresh(&path, SystemTime::now()));
    }

    #[test]
    fn cache_marker_missing_not_fresh() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("latest_version");
        assert!(!is_cache_fresh(&path, SystemTime::now()));
    }

    // -- needs_update comparison --

    #[test]
    fn needs_update_when_latest_is_newer() {
        let current = semver::Version::new(0, 3, 0);
        let latest = semver::Version::new(0, 4, 0);
        assert!(needs_update(&current, &latest));
    }

    #[test]
    fn no_update_when_equal() {
        let current = semver::Version::new(0, 3, 0);
        let latest = semver::Version::new(0, 3, 0);
        assert!(!needs_update(&current, &latest));
    }

    #[test]
    fn no_update_when_current_is_newer() {
        let current = semver::Version::new(0, 5, 0);
        let latest = semver::Version::new(0, 4, 0);
        assert!(!needs_update(&current, &latest));
    }

    // -- cache_marker_path --

    #[test]
    fn cache_marker_path_returns_some() {
        assert!(cache_marker_path().is_some());
    }
}
