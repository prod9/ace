mod config;
mod diff;
mod maverick;
mod fmt;
mod import;
mod main;
mod mcp;
mod paths;
mod pull;
mod school;
mod setup;
mod yolo;

use clap::{Parser, Subcommand};

use crate::ace::{Ace, IoError};
use crate::config::ConfigError;
use crate::state::actions::import_skill::ImportError;
use crate::state::actions::mcp_register::McpRegisterError;
use crate::state::actions::prepare::PrepareError;
use crate::state::actions::school_init::SchoolInitError;
use crate::state::actions::school_update::SchoolUpdateError;
use crate::state::actions::setup::SetupError;
use crate::git::GitError;

#[derive(Parser)]
#[command(
    name = "ace",
    about = "AI Coding Environment",
    version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("ACE_GIT_HASH"), ")"),
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Machine-readable output (no colors, no spinners, no logo)
    #[arg(long, global = true)]
    pub porcelain: bool,

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
    /// Show uncommitted changes in the school cache
    Diff,
    /// Format ace.toml / school.toml (pretty-print, strip empties)
    Fmt,
    /// Format ace.toml / school.toml (alias for fmt)
    Format,
    /// Print effective configuration
    Config,
    /// Print resolved filesystem paths ACE uses
    Paths {
        /// Print only this key (e.g. "school", "config.user")
        key: Option<String>,
    },
    /// Import a skill from an external repository into the school
    Import {
        /// Skill source (owner/repo or URL)
        source: String,
        /// Specific skill name within the repo
        #[arg(long)]
        skill: Option<String>,
    },
    /// Manage MCP server registrations
    Mcp {
        #[command(subcommand)]
        command: Option<mcp::Command>,
    },
    /// Manage schools
    School {
        #[command(subcommand)]
        command: school::Command,
    },
    /// Fetch latest school changes (force, ignoring cooldown)
    Pull,
    /// Enable auto trust mode (AI decides which actions need approval)
    Auto,
    /// Enable yolo trust mode (skip all permission prompts)
    Yolo,
    /// 🛩️
    #[command(hide = true)]
    Maverick,
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
    McpRegister(#[from] McpRegisterError),
    #[error("{0}")]
    Import(#[from] ImportError),
    #[error("{0}")]
    SchoolInit(#[from] SchoolInitError),
    #[error("{0}")]
    SchoolUpdate(#[from] SchoolUpdateError),
    #[error("{0}")]
    Git(#[from] GitError),
    #[error("{0}")]
    Prompt(#[from] IoError),
    #[error("{0}")]
    Other(String),
}

pub async fn run(ace: &mut Ace, cli: Cli) {
    match cli.command {
        Some(Command::Setup { specifier }) => setup::run(ace, specifier.as_deref()).await,
        Some(Command::Import { source, skill }) => import::run(ace, &source, skill.as_deref()),
        Some(Command::Diff) => diff::run(ace).await,
        Some(Command::Fmt) | Some(Command::Format) => fmt::run(ace),
        Some(Command::Config) => config::run(ace).await,
        Some(Command::Paths { key }) => paths::run(ace, key.as_deref()).await,
        Some(Command::Mcp { command }) => mcp::run(ace, command),
        Some(Command::School { command }) => school::run(ace, command).await,
        Some(Command::Pull) => pull::run(ace),
        Some(Command::Auto) => yolo::run(ace, crate::config::ace_toml::Trust::Auto),
        Some(Command::Yolo) => yolo::run(ace, crate::config::ace_toml::Trust::Yolo),
        Some(Command::Maverick) => maverick::run(ace),
        None => main::run(ace, cli.backend_args).await,
    }
}

fn exit_on_err(ace: &mut Ace, result: Result<(), CmdError>) {
    if let Err(e) = result {
        ace.error(&e.to_string());
        std::process::exit(1);
    }
}
