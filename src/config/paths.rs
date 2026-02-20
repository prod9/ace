use std::path::PathBuf;

pub struct Paths {
    pub config_user: Option<PathBuf>,
    pub config_local: PathBuf,
    pub config_project: PathBuf,
    pub school_source: Option<String>,
    pub school_cache: Option<PathBuf>,
    pub school_root: Option<PathBuf>,
}

pub fn resolve(
    project_dir: &std::path::Path,
    school_specifier: Option<&str>,
) -> Result<Paths, ParseError> {
    let config_user = config_dir().map(|d| d.join("ace").join("config.toml"));
    let config_local = project_dir.join("ace.local.toml");
    let config_project = project_dir.join("ace.toml");

    let (school_cache, school_root) = match school_specifier {
        Some(spec) => {
            let (source, path) = parse_specifier(spec)?;
            if source == "." {
                let root = match path {
                    Some(p) => project_dir.join(p),
                    None => project_dir.to_path_buf(),
                };
                (None, Some(root))
            } else {
                let cache = cache_dir().map(|d| d.join("ace").join(&source));
                let root = cache.as_ref().map(|c| match path {
                    Some(p) => c.join(p),
                    None => c.clone(),
                });
                (cache, root)
            }
        }
        None => (None, None),
    };

    Ok(Paths {
        config_user,
        config_local,
        config_project,
        school_source: school_specifier.map(String::from),
        school_cache,
        school_root,
    })
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    TraversalInSource(String),
    TraversalInPath(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TraversalInSource(s) => write!(f, "path traversal in source: {s}"),
            Self::TraversalInPath(s) => write!(f, "path traversal in path: {s}"),
        }
    }
}

/// Parse "source:path" specifier into (source, optional path).
fn parse_specifier(spec: &str) -> Result<(String, Option<&str>), ParseError> {
    let (source, path) = match spec.split_once(':') {
        Some((source, path)) => {
            let path = path.trim_start_matches('/');
            (source, if path.is_empty() { None } else { Some(path) })
        }
        None => (spec, None),
    };

    if source != "." && has_traversal(source) {
        return Err(ParseError::TraversalInSource(source.to_string()));
    }
    if let Some(p) = path {
        if has_traversal(p) {
            return Err(ParseError::TraversalInPath(p.to_string()));
        }
    }

    Ok((source.to_string(), path))
}

fn has_traversal(s: &str) -> bool {
    s.split('/').any(|seg| seg == "." || seg == "..")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn config_dir() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let home = home_dir().map(|h| h.join(".config"));
    xdg.or_else(|| home)
}

fn cache_dir() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from);
    let home = home_dir().map(|h| h.join(".cache"));
    xdg.or_else(|| home)
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
    fn resolve_config_paths() {
        let project = Path::new("/tmp/myproject");
        let p = resolve(project, None).unwrap();

        assert_eq!(p.config_local, PathBuf::from("/tmp/myproject/ace.local.toml"));
        assert_eq!(p.config_project, PathBuf::from("/tmp/myproject/ace.toml"));
        assert!(p.school_source.is_none());
        assert!(p.school_cache.is_none());
        assert!(p.school_root.is_none());
    }

    #[test]
    fn resolve_embedded_school() {
        let cases: &[(&str, &str)] = &[
            (".:/school", "/tmp/myproject/school"),
            (".:school", "/tmp/myproject/school"),
            (".", "/tmp/myproject"),
        ];

        for (spec, expected_root) in cases {
            let p = resolve(Path::new("/tmp/myproject"), Some(spec)).unwrap();
            assert!(p.school_cache.is_none(), "embedded school should have no cache for {spec:?}");
            assert_eq!(
                p.school_root,
                Some(PathBuf::from(expected_root)),
                "root mismatch for {spec:?}"
            );
        }
    }

    #[test]
    fn resolve_remote_school() {
        let cases: &[(&str, &str, &str)] = &[
            ("prod9/school", "ace/prod9/school", "ace/prod9/school"),
            ("prod9/mono:school", "ace/prod9/mono", "ace/prod9/mono/school"),
        ];

        for (spec, cache_suffix, root_suffix) in cases {
            let p = resolve(Path::new("/tmp/myproject"), Some(spec)).unwrap();

            let cache = p.school_cache.as_ref().unwrap_or_else(|| {
                panic!("school_cache should be Some for {spec:?}")
            });
            assert!(cache.ends_with(cache_suffix), "cache {cache:?} should end with {cache_suffix:?}");

            let root = p.school_root.as_ref().unwrap_or_else(|| {
                panic!("school_root should be Some for {spec:?}")
            });
            assert!(root.ends_with(root_suffix), "root {root:?} should end with {root_suffix:?}");
        }
    }

    #[test]
    fn resolve_rejects_traversal() {
        let project = Path::new("/tmp/myproject");
        let result = resolve(project, Some("owner/repo:../secret"));
        assert!(result.is_err());
    }
}
