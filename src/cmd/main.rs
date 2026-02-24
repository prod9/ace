use crate::ace::Ace;
use crate::config;
use crate::state::actions::exec::Exec;
use crate::state::actions::prepare::Prepare;
use crate::state::prompt::build_session_prompt;
use crate::state::State;

use super::CmdError;

pub async fn run(ace: &mut Ace, backend_args: Vec<String>) {
    let result = run_inner(ace, backend_args).await;
    super::exit_on_err(ace, result);
}

async fn run_inner(ace: &mut Ace, backend_args: Vec<String>) -> Result<(), CmdError> {
    let project_dir = std::env::current_dir()?;
    let paths = config::paths::resolve(&project_dir)?;
    let mut tree = config::tree::Tree::load(&paths)?;

    // Pass 1: resolve school specifier to know which school.toml to load.
    let specifier = State::resolve_specifier(&tree)
        .ok_or(CmdError::NoSchool)?;

    // Prepare (install/update/link) needs a preliminary backend for skills_dir.
    let preliminary_backend = tree.local.backend
        .or(tree.project.backend)
        .or(tree.user.backend)
        .unwrap_or_default();

    let mut preliminary_ace = Ace::new(Ace::term_sink());
    let prepare_result = (Prepare {
        specifier: &specifier,
        project_dir: &project_dir,
        skills_dir: preliminary_backend.skills_dir(),
    })
    .run(&mut preliminary_ace)
    .await?;

    // Pass 2: load school.toml, feed backend into tree, full resolve.
    let school_paths = config::school_paths::resolve(&project_dir, &specifier)?;
    let school_toml_path = school_paths.root.join("school.toml");
    let school_toml = config::school_toml::load(&school_toml_path)
        .map_err(|e| CmdError::Other(format!("{}: {e}", school_toml_path.display())))?;
    tree.school_backend = school_toml.backend;

    let state = State::resolve(tree);
    *ace = Ace::with_state(state, Ace::term_sink());

    let skills_dir = project_dir.join(ace.state.backend.skills_dir());
    let session_prompt = build_session_prompt(
        &school_toml.name,
        &school_toml.session_prompt,
        &ace.state.session_prompt,
        &skills_dir,
        &prepare_result.changes,
    );

    Exec {
        backend: ace.state.backend,
        session_prompt,
        project_dir: project_dir.clone(),
        env: ace.state.env.clone(),
        backend_args,
    }
    .run(ace)?;

    Ok(())
}
