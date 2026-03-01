mod auth;
mod config;
mod diff;
mod fmt;
mod import;
mod main;
mod paths;
mod school;
mod setup;

use clap::{Parser, Subcommand};

use crate::ace::Ace;
use crate::config::ConfigError;
use crate::state::actions::import_skill::ImportError;
use crate::state::actions::prepare::PrepareError;
use crate::state::actions::school_init::SchoolInitError;
use crate::state::actions::school_propose::SchoolProposeError;
use crate::state::actions::school_update::SchoolUpdateError;
use crate::state::actions::setup::SetupError;
use crate::git::GitError;
use crate::term_ui::TermError;

#[derive(Parser)]
#[command(
    name = "ace",
    about = "AI Coding Environment",
    version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("ACE_GIT_HASH"), ")"),
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Extra arguments passed through to the backend (claude/opencode), after --
    #[arg(last = true)]
    backend_args: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Set up a school (clone + auth + config)
    Setup {
        /// School specifier (owner/repo). Omit to link a cached school.
        specifier: Option<String>,
    },
    /// Re-authenticate a service
    Auth {
        /// Service name to authenticate
        name: String,
    },
    /// Show uncommitted changes in the school cache
    Diff,
    /// Format ace.toml / school.toml (pretty-print, strip empties)
    Fmt,
    /// Format ace.toml / school.toml (alias for fmt)
    Format,
    /// Print effective configuration
    Config,
    /// Print resolved filesystem paths ACE uses
    Paths,
    /// Import a skill from an external repository into the school
    Import {
        /// Skill source (owner/repo or URL)
        source: String,
        /// Specific skill name within the repo
        #[arg(long)]
        skill: Option<String>,
    },
    /// Manage schools
    School {
        #[command(subcommand)]
        command: school::Command,
    },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CmdError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0}")]
    Setup(#[from] SetupError),
    #[error("{0}")]
    Prepare(#[from] PrepareError),
    #[error("{0}")]
    Import(#[from] ImportError),
    #[error("{0}")]
    SchoolInit(#[from] SchoolInitError),
    #[error("{0}")]
    SchoolPropose(#[from] SchoolProposeError),
    #[error("{0}")]
    SchoolUpdate(#[from] SchoolUpdateError),
    #[error("{0}")]
    Git(#[from] GitError),
    #[error("{0}")]
    Tui(#[from] TermError),
    #[error("no school configured, run `ace setup`")]
    NoSchool,
    #[error("{0}")]
    Other(String),
}

pub async fn run(ace: &mut Ace, cli: Cli) {
    match cli.command {
        Some(Command::Setup { specifier }) => setup::run(ace, specifier.as_deref()).await,
        Some(Command::Auth { name }) => auth::run(ace, &name).await,
        Some(Command::Import { source, skill }) => import::run(ace, &source, skill.as_deref()),
        Some(Command::Diff) => diff::run(ace).await,
        Some(Command::Fmt) | Some(Command::Format) => fmt::run(ace),
        Some(Command::Config) => config::run(ace).await,
        Some(Command::Paths) => paths::run(ace).await,
        Some(Command::School { command }) => school::run(ace, command).await,
        None => main::run(ace, cli.backend_args).await,
    }
}

fn exit_on_err(ace: &mut Ace, result: Result<(), CmdError>) {
    if let Err(e) = result {
        ace.error(&e.to_string());
        std::process::exit(1);
    }
}
