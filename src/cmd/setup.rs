use crate::ace::Ace;
use crate::config::index_toml;
use crate::state::actions::setup::{Setup, SetupError};
use crate::term_ui;

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Setup(#[from] SetupError),
    #[error("{0}")]
    Cache(String),
    #[error("{0}")]
    Tui(#[from] term_ui::TermError),
}

pub async fn run(ace: &mut Ace, specifier: Option<&str>) {
    if let Err(e) = run_inner(ace, specifier).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run_inner(ace: &mut Ace, specifier: Option<&str>) -> Result<(), RunError> {
    let project_dir = std::env::current_dir()?;

    let resolved = match specifier {
        Some(s) => s.to_string(),
        None => resolve_from_cache().map_err(RunError::Cache)?,
    };

    let mut session = ace.session();
    Setup {
        specifier: &resolved,
        project_dir: &project_dir,
    }
    .run(&mut session)
    .await?;

    println!("Setup complete.");
    Ok(())
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
            let choice = term_ui::select("Select school:", specs)
                .map_err(|e| e.to_string())?;
            Ok(choice)
        }
    }
}
