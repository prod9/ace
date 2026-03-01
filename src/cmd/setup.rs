use crate::ace::Ace;
use crate::config::index_toml;
use crate::state::actions::setup::Setup;
use crate::term_ui;

use super::CmdError;

pub async fn run(ace: &mut Ace, specifier: Option<&str>) {
    let result = run_inner(ace, specifier).await;
    super::exit_on_err(ace, result);
}

async fn run_inner(ace: &mut Ace, specifier: Option<&str>) -> Result<(), CmdError> {
    let project_dir = ace.project_dir().to_path_buf();

    let resolved = match specifier {
        Some(s) => s.to_string(),
        None => resolve_from_cache()?,
    };

    Setup {
        specifier: &resolved,
        project_dir: &project_dir,
    }
    .run(ace)
    .await?;

    ace.done("Setup complete.");
    Ok(())
}

fn resolve_from_cache() -> Result<String, CmdError> {
    let index_path = index_toml::index_path()
        .map_err(|e| CmdError::Other(format!("{e}")))?;
    let index = index_toml::load(&index_path)
        .map_err(|e| CmdError::Other(format!("{e}")))?;

    let specs = index_toml::list_specifiers(&index);
    match specs.len() {
        0 => Err(CmdError::Other("no cached schools, ace setup <owner/repo>?".to_string())),
        1 => Ok(specs.into_iter().next().expect("checked len=1")),
        _ => {
            let choice = term_ui::select("Select school:", specs)?;
            Ok(choice)
        }
    }
}
