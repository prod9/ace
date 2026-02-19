/// Print resolved filesystem paths ACE uses.
///
/// Prints config file locations, school cache directories, and school roots.
/// Paths are printed regardless of whether they exist on disk.
/// Tab-separated for machine parseability.
///
/// For embedded schools (source `.`), school.cache is omitted.
use crate::ace::Ace;
use crate::config::paths;

pub async fn run(ace: &Ace) {
    let cwd = std::env::current_dir().unwrap_or_default();
    let specifier = ace.config().school_specifier.as_deref();
    let p = match paths::resolve(&cwd, specifier) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    if let Some(ref path) = p.config_user {
        println!("config.user\t{}", path.display());
    }
    println!("config.local\t{}", p.config_local.display());
    println!("config.project\t{}", p.config_project.display());

    if let Some(ref spec) = p.school_source {
        println!("school.source\t{spec}");
    }
    if let Some(ref path) = p.school_cache {
        println!("school.cache\t{}", path.display());
    }
    if let Some(ref path) = p.school_root {
        println!("school.root\t{}", path.display());
    }
}
