use std::path::PathBuf;

use super::paths::cache_dir;
use super::ConfigError;

pub struct SchoolPaths {
    pub source: String,
    pub cache: Option<PathBuf>,
    pub root: PathBuf,
}

pub fn resolve(
    project_dir: &std::path::Path,
    specifier: &str,
) -> Result<SchoolPaths, ConfigError> {
    let (source, path) = parse_specifier(specifier)?;
    let (base, cache) = if source == "." {
        (project_dir.to_path_buf(), None)
    } else {
        let cache = cache_dir()?.join("ace").join(&source);
        (cache.clone(), Some(cache))
    };
    let root = path.map(|p| base.join(p)).unwrap_or(base.clone());
    Ok(SchoolPaths {
        source: specifier.to_string(),
        cache,
        root,
    })
}

/// Parse "source:path" specifier into (source, optional path).
fn parse_specifier(spec: &str) -> Result<(String, Option<&str>), ConfigError> {
    let (source, path) = match spec.split_once(':') {
        Some((source, path)) => {
            let path = path.trim_start_matches('/');
            (source, if path.is_empty() { None } else { Some(path) })
        }
        None => (spec, None),
    };

    if source != "." && has_traversal(source) {
        return Err(ConfigError::TraversalInSource(source.to_string()));
    }

    if let Some(p) = path {
        if has_traversal(p) {
            return Err(ConfigError::TraversalInPath(p.to_string()));
        }
    }

    Ok((source.to_string(), path))
}

fn has_traversal(s: &str) -> bool {
    s.split('/').any(|seg| seg == "." || seg == "..")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_specifier_valid() {
        let cases: &[(&str, &str, Option<&str>)] = &[
            ("prod9/school", "prod9/school", None),
            ("prod9/mono:school", "prod9/mono", Some("school")),
            (".", ".", None),
            (".:/school", ".", Some("school")),
            (".:school", ".", Some("school")),
            ("prod9/mono:/deep/path", "prod9/mono", Some("deep/path")),
            ("owner/repo:", "owner/repo", None),
        ];

        for (input, expected_source, expected_path) in cases {
            let (source, path) = parse_specifier(input).unwrap_or_else(|e| {
                panic!("parse_specifier({input:?}) should succeed but got: {e}")
            });
            assert_eq!(source, *expected_source, "source mismatch for {input:?}");
            assert_eq!(path, *expected_path, "path mismatch for {input:?}");
        }
    }

    #[test]
    fn parse_specifier_rejects_traversal() {
        let cases: &[&str] = &[
            "../escape",
            "owner/../etc",
            "./sneaky",
            "owner/repo:../secret",
            "owner/repo:foo/../../etc",
            "owner/repo:./here",
            "owner/repo:foo/./bar",
            "../../../etc/passwd",
            "owner/repo:../../../etc/passwd",
        ];

        for input in cases {
            let result = parse_specifier(input);
            assert!(result.is_err(), "parse_specifier({input:?}) should fail but got: {result:?}");
        }
    }

    #[test]
    fn resolve_embedded() {
        let cases: &[(&str, &str)] = &[
            (".:/school", "/tmp/myproject/school"),
            (".:school", "/tmp/myproject/school"),
            (".", "/tmp/myproject"),
        ];

        for (spec, expected_root) in cases {
            let p = resolve(Path::new("/tmp/myproject"), spec)
                .expect("resolve should succeed for embedded spec");
            assert!(p.cache.is_none(), "embedded school should have no cache for {spec:?}");
            assert_eq!(
                p.root,
                PathBuf::from(expected_root),
                "root mismatch for {spec:?}"
            );
        }
    }

    #[test]
    fn resolve_remote() {
        let cases: &[(&str, &str, &str)] = &[
            ("prod9/school", "ace/prod9/school", "ace/prod9/school"),
            ("prod9/mono:school", "ace/prod9/mono", "ace/prod9/mono/school"),
        ];

        for (spec, cache_suffix, root_suffix) in cases {
            let p = resolve(Path::new("/tmp/myproject"), spec)
                .expect("resolve should succeed for remote spec");

            let cache = p.cache.as_ref()
                .expect("cache should be Some for remote spec");
            assert!(cache.ends_with(cache_suffix), "cache {cache:?} should end with {cache_suffix:?}");
            assert!(p.root.ends_with(root_suffix), "root {:?} should end with {root_suffix:?}", p.root);
        }
    }

    #[test]
    fn resolve_rejects_traversal() {
        let project = Path::new("/tmp/myproject");
        let result = resolve(project, "owner/repo:../secret");
        assert!(result.is_err());
    }
}
