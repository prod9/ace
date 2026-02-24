use clap::Subcommand;

use crate::ace::Ace;
use crate::state::actions::school_init::{SchoolInit, SchoolInitError};
use crate::state::actions::school_propose::{SchoolPropose, SchoolProposeError};
use crate::state::actions::school_update::{SchoolUpdate, SchoolUpdateError, SchoolUpdateResult};
use crate::term_ui::{TermError, Tui, Workflow};

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new school repository
    Init {
        /// School display name
        #[arg(long)]
        name: Option<String>,
        /// Overwrite existing school.toml
        #[arg(long)]
        force: bool,
    },
    /// Propose local school changes back to upstream
    #[clap(alias = "pr")]
    Propose,
    /// Re-fetch all imported skills from their sources
    Update,
}

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Load(#[from] crate::ace::LoadError),
    #[error("{0}")]
    Init(#[from] SchoolInitError),
    #[error("{0}")]
    Propose(#[from] SchoolProposeError),
    #[error("{0}")]
    SchoolUpdate(#[from] SchoolUpdateError),
    #[error("{0}")]
    Resolve(#[from] crate::config::school_paths::ResolveError),
    #[error("{0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("{0}")]
    Tui(#[from] TermError),
    #[error("no school linked, run ace setup first")]
    NoSchool,
    #[error("{0}")]
    Token(String),
}

pub async fn run(ace: &mut Ace, command: Command) {
    match command {
        Command::Init { name, force } => {
            if let Err(e) = run_init(ace, name, force) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Command::Propose => {
            if let Err(e) = run_propose() {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Command::Update => {
            if let Err(e) = run_update() {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn run_init(ace: &mut Ace, name: Option<String>, force: bool) -> Result<(), RunError> {
    match name {
        Some(name) => {
            let project_dir = std::env::current_dir()?;
            let mut session = ace.session();
            SchoolInit {
                name: &name,
                project_dir: &project_dir,
                force,
            }
            .run(&mut session)?;
        }
        None => {
            Tui::new(ace).run(Workflow::SchoolInit { force })?;
        }
    }
    Ok(())
}

fn run_propose() -> Result<(), RunError> {
    let project_dir = std::env::current_dir()?;
    let mut ace = crate::ace::Ace::load(&project_dir, crate::ace::Ace::term_sink())?;

    let specifier = ace.state().school_specifier.clone()
        .ok_or(RunError::NoSchool)?;

    let repo_key = specifier.split_once(':').map_or(specifier.as_str(), |(repo, _)| repo);
    let token = load_github_token(repo_key).map_err(RunError::Token)?;

    let mut session = ace.session();
    let url = SchoolPropose {
        project_dir: &project_dir,
        token: &token,
    }
    .run(&mut session)?;

    println!("PR created: {url}");
    Ok(())
}

fn run_update() -> Result<(), RunError> {
    let school_root = resolve_school_root()?;

    let mut ace = crate::ace::Ace::new(crate::ace::Ace::term_sink());
    let mut session = ace.session();

    let result = SchoolUpdate { school_root: &school_root }.run(&mut session)?;
    match result {
        SchoolUpdateResult::NoImports => eprintln!("no imports to update"),
        SchoolUpdateResult::Updated { .. } => {}
    }
    Ok(())
}

fn resolve_school_root() -> Result<std::path::PathBuf, RunError> {
    let cwd = std::env::current_dir()?;

    if cwd.join("school.toml").exists() {
        return Ok(cwd);
    }

    let ace_toml_path = cwd.join("ace.toml");
    if ace_toml_path.exists() {
        let ace = crate::config::ace_toml::load(&ace_toml_path)?;
        let paths = crate::config::school_paths::resolve(&cwd, &ace.school)?;
        return Ok(paths.root);
    }

    Err(RunError::NoSchool)
}

fn load_github_token(repo_key: &str) -> Result<String, String> {
    let path = crate::config::user_config::default_path()
        .ok_or("cannot determine config dir")?;
    let config = crate::config::user_config::load(&path)
        .map_err(|e| format!("load config: {e}"))?;

    let school = config
        .get(repo_key)
        .ok_or(format!("no config for school {repo_key}, run ace setup"))?;
    let github = school
        .services
        .get("github")
        .ok_or(format!("no github token for {repo_key}, run ace auth"))?;

    github
        .token
        .clone()
        .ok_or(format!("github token empty for {repo_key}"))
}
