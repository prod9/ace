/// Simple glob matching for skill name patterns.
///
/// Supports `*` as a wildcard matching zero or more characters.
/// No `?`, `**`, or character classes.
///
/// Examples:
/// - `"*"` matches everything
/// - `"frontend-*"` matches `"frontend-design"`, `"frontend-review"`
/// - `"*-coding"` matches `"go-coding"`, `"rust-coding"`
/// - `"*-design-*"` matches `"frontend-design-system"`
/// - `"go-coding"` matches only `"go-coding"` (exact)

/// Returns true if `pattern` contains a `*` wildcard.
pub fn is_glob(pattern: &str) -> bool {
    pattern.contains('*')
}

/// Match `name` against a glob `pattern`.
///
/// Splits the pattern on `*` and checks that all literal parts appear
/// in order within the name. Leading/trailing `*` allow prefix/suffix
/// flexibility.
pub fn glob_match(pattern: &str, name: &str) -> bool {
    if !is_glob(pattern) {
        return pattern == name;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        // First part must be a prefix if pattern doesn't start with *
        if i == 0 {
            let Some(rest) = name.strip_prefix(part) else {
                return false;
            };
            pos = name.len() - rest.len();
            continue;
        }

        // Last part must be a suffix if pattern doesn't end with *
        if i == parts.len() - 1 {
            return name[pos..].ends_with(part);
        }

        // Middle parts: find next occurrence after current position
        let Some(found) = name[pos..].find(part) else {
            return false;
        };
        pos += found + part.len();
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_matches_everything() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*", ""));
    }

    #[test]
    fn exact_match_without_glob() {
        assert!(glob_match("go-coding", "go-coding"));
        assert!(!glob_match("go-coding", "rust-coding"));
    }

    #[test]
    fn prefix_glob() {
        assert!(glob_match("frontend-*", "frontend-design"));
        assert!(glob_match("frontend-*", "frontend-"));
        assert!(!glob_match("frontend-*", "backend-design"));
    }

    #[test]
    fn suffix_glob() {
        assert!(glob_match("*-coding", "go-coding"));
        assert!(glob_match("*-coding", "rust-coding"));
        assert!(!glob_match("*-coding", "go-review"));
    }

    #[test]
    fn middle_glob() {
        assert!(glob_match("front*end", "frontend"));
        assert!(glob_match("front*end", "front-and-backend"));
        assert!(!glob_match("front*end", "frontend-design"));
    }

    #[test]
    fn multiple_globs() {
        assert!(glob_match("*-design-*", "frontend-design-system"));
        assert!(glob_match("*-design-*", "x-design-y"));
        assert!(!glob_match("*-design-*", "frontend-review-system"));
    }

    #[test]
    fn is_glob_detects_star() {
        assert!(is_glob("*"));
        assert!(is_glob("frontend-*"));
        assert!(!is_glob("go-coding"));
    }

    #[test]
    fn consecutive_stars() {
        assert!(glob_match("**", "anything"));
        assert!(glob_match("a**b", "axyzb"));
    }

    #[test]
    fn no_match_empty_name_with_literal() {
        assert!(!glob_match("abc", ""));
        assert!(!glob_match("*-coding", ""));
    }
}
