use crate::ace::Ace;
use crate::config::index_toml;
use crate::state::actions::setup::Setup;

pub async fn run(ace: &mut Ace, specifier: Option<&str>) {
    let project_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let resolved = match specifier {
        Some(s) => s.to_string(),
        None => match resolve_from_cache() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
    };

    let setup = Setup {
        specifier: &resolved,
        project_dir: &project_dir,
    };

    let mut session = ace.session();
    match setup.run(&mut session).await {
        Ok(()) => println!("Setup complete."),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn resolve_from_cache() -> Result<String, String> {
    let index_path = index_toml::index_path()
        .map_err(|e| format!("{e}"))?;
    let index = index_toml::load(&index_path)
        .map_err(|e| format!("{e}"))?;

    let specs = index_toml::list_specifiers(&index);
    match specs.len() {
        0 => Err("no cached schools, ace setup <owner/repo>?".to_string()),
        1 => Ok(specs.into_iter().next().expect("checked len=1")),
        _ => {
            // TODO: TUI school picker
            eprintln!("Multiple schools cached:");
            for s in &specs {
                eprintln!("  {s}");
            }
            Err("multiple schools cached, specify one: ace setup <owner/repo>".to_string())
        }
    }
}
