use crate::ace::Ace;
use crate::config;
use crate::state::actions::exec::{Exec, ExecError};
use crate::state::actions::prepare::Prepare;
use crate::state::actions::setup::SetupError;
use crate::state::prompt::build_session_prompt;

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Load(#[from] crate::ace::LoadError),
    #[error("no school configured, run `ace setup`")]
    NoSchool,
    #[error("{0}")]
    Prepare(#[from] SetupError),
    #[error("{0}")]
    SchoolPath(#[from] config::school_paths::ResolveError),
    #[error("school.toml: {0}")]
    SchoolToml(#[from] config::ConfigError),
    #[error("{0}")]
    Exec(#[from] ExecError),
}

pub async fn run(ace: &mut Ace) {
    if let Err(e) = run_inner(ace).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run_inner(ace: &mut Ace) -> Result<(), RunError> {
    let project_dir = std::env::current_dir()?;
    *ace = Ace::load(&project_dir)?;

    let mut session = ace.session();

    let specifier = session.state.school_specifier.clone()
        .ok_or(RunError::NoSchool)?;

    let backend = session.state.backend;

    (Prepare {
        specifier: &specifier,
        project_dir: &project_dir,
        skills_dir: backend.skills_dir(),
    })
    .run(&mut session)
    .await?;

    let school_paths = config::school_paths::resolve(&project_dir, &specifier)?;
    let school_toml_path = school_paths.root.join("school.toml");
    let school_toml = config::school_toml::load(&school_toml_path)?;

    let session_prompt = build_session_prompt(
        &school_toml.school.name,
        school_toml.school.description.as_deref(),
        &school_toml.school.session_prompt,
        &session.state.session_prompt,
    );

    let env = session.state.env.clone();
    Exec {
        backend,
        session_prompt,
        project_dir: project_dir.clone(),
        env,
    }
    .run(&mut session)?;

    Ok(())
}
