use crate::ace::Ace;
use crate::config::{paths, school_paths};

use super::CmdError;

pub async fn run(ace: &mut Ace, key: Option<&str>) {
    let result = run_inner(ace, key);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace, key: Option<&str>) -> Result<(), CmdError> {
    ace.require_state()?;
    let p = paths::resolve(ace.project_dir())?;

    let all = build_paths(ace, &p)?;

    match key {
        Some(k) => {
            let Some(value) = lookup_key(&all, k) else {
                return Err(CmdError::Other(format!("unknown key: {k}")));
            };
            ace.data(value);
        }
        None => {
            for (k, v) in &all {
                ace.data(&format!("{k}\t{v}"));
            }
        }
    }

    Ok(())
}

fn build_paths(
    ace: &Ace,
    p: &paths::AcePaths,
) -> Result<Vec<(String, String)>, CmdError> {
    let mut out = Vec::new();

    out.push(("config.user".into(), p.user.display().to_string()));
    out.push(("config.local".into(), p.local.display().to_string()));
    out.push(("config.project".into(), p.project.display().to_string()));
    out.push(("cache.root".into(), p.cache.display().to_string()));

    if let Some(spec) = ace.state().school_specifier.as_deref() {
        let sp = school_paths::resolve(ace.project_dir(), spec)?;

        out.push(("school.source".into(), sp.source));
        if let Some(ref path) = sp.cache {
            out.push(("school.cache".into(), path.display().to_string()));
        }
        out.push(("school.root".into(), sp.root.display().to_string()));
    }

    Ok(out)
}

/// Lookup by exact key or shorthand alias.
fn lookup_key<'a>(all: &'a [(String, String)], key: &str) -> Option<&'a str> {
    // Exact match first
    if let Some((_, v)) = all.iter().find(|(k, _)| k == key) {
        return Some(v);
    }

    // Shorthand aliases
    let full_key = match key {
        "school" => "school.cache",
        "cache" => "cache.root",
        "school-cache" => "school.cache",
        "root" => "school.root",
        "user" => "config.user",
        "local" => "config.local",
        "project" => "config.project",
        _ => return None,
    };

    all.iter().find(|(k, _)| k == full_key).map(|(_, v)| v.as_str())
}
