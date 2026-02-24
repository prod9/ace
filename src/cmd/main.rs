use crate::ace::Ace;
use crate::config;
use crate::state::actions::exec::{Exec, ExecError};
use crate::state::actions::prepare::Prepare;
use crate::state::actions::setup::SetupError;
use crate::state::prompt::build_session_prompt;
use crate::state::State;

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Path(#[from] config::paths::PathError),
    #[error("{0}")]
    Tree(#[from] config::tree::LoadError),
    #[error("no school configured, run `ace setup`")]
    NoSchool,
    #[error("{0}")]
    Prepare(#[from] SetupError),
    #[error("{0}")]
    SchoolPath(#[from] config::school_paths::ResolveError),
    #[error("{0}")]
    SchoolToml(String),
    #[error("{0}")]
    Exec(#[from] ExecError),
}

pub async fn run(ace: &mut Ace, backend_args: Vec<String>) {
    if let Err(e) = run_inner(ace, backend_args).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run_inner(ace: &mut Ace, backend_args: Vec<String>) -> Result<(), RunError> {
    let project_dir = std::env::current_dir()?;
    let paths = config::paths::resolve(&project_dir)?;
    let mut tree = config::tree::Tree::load(&paths)?;

    // Pass 1: resolve school specifier to know which school.toml to load.
    let specifier = State::resolve_specifier(&tree)
        .ok_or(RunError::NoSchool)?;

    // Prepare (install/update/link) needs a preliminary backend for skills_dir.
    let preliminary_backend = tree.local.backend
        .or(tree.project.backend)
        .or(tree.user.backend)
        .unwrap_or_default();

    let mut preliminary_state = State::empty();
    let mut session = crate::session::Session { state: &mut preliminary_state };
    (Prepare {
        specifier: &specifier,
        project_dir: &project_dir,
        skills_dir: preliminary_backend.skills_dir(),
    })
    .run(&mut session)
    .await?;

    // Pass 2: load school.toml, feed backend into tree, full resolve.
    let school_paths = config::school_paths::resolve(&project_dir, &specifier)?;
    let school_toml_path = school_paths.root.join("school.toml");
    let school_toml = config::school_toml::load(&school_toml_path)
        .map_err(|e| RunError::SchoolToml(format!("{}: {e}", school_toml_path.display())))?;
    tree.school_backend = school_toml.school.backend;

    let state = State::resolve(tree);
    *ace = Ace::with_state(state);
    let mut session = ace.session();

    let skills_dir = project_dir.join(session.state.backend.skills_dir());
    let session_prompt = build_session_prompt(
        &school_toml.school.name,
        school_toml.school.description.as_deref(),
        &school_toml.school.session_prompt,
        &session.state.session_prompt,
        &skills_dir,
    );

    Exec {
        backend: session.state.backend,
        session_prompt,
        project_dir: project_dir.clone(),
        env: session.state.env.clone(),
        backend_args,
    }
    .run(&mut session)?;

    Ok(())
}
