use crate::ace::Ace;
use crate::config::index_toml;
use crate::state::actions::gitignore::UpdateGitignore;
use crate::state::actions::setup::Setup;
use crate::templates;

use super::CmdError;

pub async fn run(ace: &mut Ace, specifier: Option<&str>) {
    let result = run_inner(ace, specifier).await;
    super::exit_on_err(ace, result);
}

async fn run_inner(ace: &mut Ace, specifier: Option<&str>) -> Result<(), CmdError> {
    let project_dir = ace.project_dir().to_path_buf();

    let resolved = match specifier {
        Some(s) => s.to_string(),
        None => resolve_from_cache(ace)?,
    };

    Setup {
        specifier: &resolved,
        project_dir: &project_dir,
    }
    .run(ace)?;

    // Prepare school (install/update/link + MCP).
    ace.require_state()?;
    super::main::prepare_school(ace, &resolved).await?;

    // Post-prepare setup: gitignore and instructions file.
    let backend = ace.state().backend;

    UpdateGitignore {
        project_dir: &project_dir,
        backend_dir: backend.backend_dir(),
    }
    .run(ace)
    .map_err(|e| CmdError::Other(format!("gitignore: {e}")))?;

    let instructions = project_dir.join(backend.instructions_file());
    if !instructions.exists() {
        let school_name = ace.state().school.as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_default();

        let backend_dir_name = backend.backend_dir();
        let tpl = templates::Template::parse(templates::builtins::PROJECT_CLAUDE_MD);
        let content = tpl.substitute(&std::collections::HashMap::from([
            ("school_name".to_string(), school_name),
            ("backend_dir".to_string(), backend_dir_name.to_string()),
        ]));

        std::fs::write(&instructions, content)?;
        ace.done(&format!("Created {}", backend.instructions_file()));
    }

    ace.done("Setup complete.");
    Ok(())
}

fn resolve_from_cache(ace: &mut Ace) -> Result<String, CmdError> {
    let index_path = index_toml::index_path()
        .map_err(|e| CmdError::Other(format!("{e}")))?;
    let index = index_toml::load(&index_path)
        .map_err(|e| CmdError::Other(format!("{e}")))?;

    let specs = index_toml::list_specifiers(&index);
    match specs.len() {
        0 => Err(CmdError::Other("no cached schools, ace setup <owner/repo>?".to_string())),
        1 => Ok(specs.into_iter().next().expect("checked len=1")),
        _ => {
            let choice = ace.prompt_select("Select school:", specs)?;
            Ok(choice)
        }
    }
}
