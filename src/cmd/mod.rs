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
mod upgrade;
mod yolo;

use clap::{Parser, Subcommand};

use crate::ace::{Ace, IoError};
use crate::config::{ConfigError, Scope};
use crate::config::backend::Backend;
use crate::state::actions::import_skill::ImportError;
use crate::state::actions::register_mcp::McpRegisterError;
use crate::state::actions::prepare_school::PrepareError;
use crate::state::actions::init_school::InitSchoolError;
use crate::state::actions::update_school::UpdateSchoolError;
use crate::state::actions::setup_project::SetupError;
use crate::git::GitError;

#[derive(Parser)]
#[command(
    name = "ace",
    about = "Augmented Coding Environment",
    version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("ACE_GIT_HASH"), ")"),
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Override the configured backend for this command invocation
    #[arg(short = 'b', long, global = true, value_enum)]
    backend: Option<Backend>,

    /// Shortcut for `--backend claude`
    #[arg(long, global = true)]
    claude: bool,

    /// Shortcut for `--backend codex`
    #[arg(long, global = true)]
    codex: bool,

    /// Shortcut for `--backend flaude`
    #[arg(long, global = true)]
    flaude: bool,

    /// Write to user-level config (~/.config/ace/ace.toml)
    #[arg(long, global = true)]
    user: bool,

    /// Alias for --user
    #[arg(long = "global", global = true, hide = true)]
    global_alias: bool,

    /// Write to project config (ace.toml)
    #[arg(long, global = true)]
    project: bool,

    /// Write to local config (ace.local.toml)
    #[arg(long, global = true)]
    local: bool,

    /// Machine-readable output (no colors, no spinners, no logo)
    #[arg(long, global = true)]
    pub porcelain: bool,

    /// Extra arguments passed through to the backend (claude/codex), after --
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
    /// Print effective configuration, or get/set individual keys
    Config {
        #[command(subcommand)]
        command: Option<config::Command>,
    },
    /// Print resolved filesystem paths ACE uses
    Paths {
        /// Print only this key (e.g. "project", "cache", "school")
        key: Option<String>,
    },
    /// Import a skill from an external repository into the school
    Import {
        /// Skill source (owner/repo or URL)
        source: String,
        /// Specific skill name or glob pattern (e.g. "frontend-*")
        #[arg(long)]
        skill: Option<String>,
        /// Import all skills from the source (equivalent to --skill "*")
        #[arg(long)]
        all: bool,
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
    /// Start a fresh session (skip auto-resume)
    New,
    /// Enable auto trust mode (AI decides which actions need approval)
    Auto,
    /// Enable yolo trust mode (skip all permission prompts)
    Yolo,
    /// Check for updates and upgrade ACE
    Upgrade {
        /// Suppress all output (used by background spawn)
        #[arg(long)]
        silent: bool,
        /// Reinstall even if at latest, or install a specific version
        #[arg(long)]
        force: bool,
        /// Specific version to install (requires --force)
        version: Option<String>,
    },
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
    InitSchool(#[from] InitSchoolError),
    #[error("{0}")]
    UpdateSchool(#[from] UpdateSchoolError),
    #[error("{0}")]
    Git(#[from] GitError),
    #[error("{0}")]
    Prompt(#[from] IoError),
    #[error("{0}")]
    Other(String),
}

pub async fn run(ace: &mut Ace, cli: Cli) {
    let backend_override = match resolve_backend_override(&cli) {
        Ok(backend) => backend,
        Err(err) => {
            exit_on_err(ace, Err(err));
            return;
        }
    };

    let scope_override = match resolve_scope_override(&cli) {
        Ok(scope) => scope,
        Err(err) => {
            exit_on_err(ace, Err(err));
            return;
        }
    };

    ace.set_backend_override(backend_override);
    ace.set_scope_override(scope_override);

    let Some(command) = cli.command else {
        return main::run(ace, cli.backend_args, true).await;
    };

    match command {
        Command::Setup { specifier } => setup::run(ace, specifier.as_deref()).await,
        Command::Import { source, skill, all } => import::run(ace, &source, skill.as_deref(), all),
        Command::Diff => diff::run(ace).await,
        Command::Fmt | Command::Format => fmt::run(ace),
        Command::Config { command } => config::run(ace, command).await,
        Command::Paths { key } => paths::run(ace, key.as_deref()).await,
        Command::Mcp { command } => mcp::run(ace, command),
        Command::School { command } => school::run(ace, command).await,
        Command::Pull => pull::run(ace),
        Command::New => main::run(ace, cli.backend_args, false).await,
        Command::Auto => yolo::run(ace, crate::config::ace_toml::Trust::Auto),
        Command::Yolo => yolo::run(ace, crate::config::ace_toml::Trust::Yolo),
        Command::Upgrade { silent, force, version } => upgrade::run(ace, silent, force, version),
        Command::Maverick => maverick::run(ace),
    }
}

fn resolve_scope_override(cli: &Cli) -> Result<Option<Scope>, CmdError> {
    let mut selected = Vec::new();

    if cli.user || cli.global_alias {
        selected.push(Scope::User);
    }
    if cli.project {
        selected.push(Scope::Project);
    }
    if cli.local {
        selected.push(Scope::Local);
    }

    selected.dedup();

    match selected.as_slice() {
        [] => Ok(None),
        [scope] => Ok(Some(*scope)),
        _ => Err(CmdError::Other(
            "cannot combine multiple scope flags (--user, --project, --local)".to_string(),
        )),
    }
}

fn resolve_backend_override(cli: &Cli) -> Result<Option<Backend>, CmdError> {
    let mut selected = Vec::new();

    if let Some(backend) = cli.backend {
        selected.push(backend);
    }
    if cli.claude {
        selected.push(Backend::Claude);
    }
    if cli.codex {
        selected.push(Backend::Codex);
    }
    if cli.flaude {
        selected.push(Backend::Flaude);
    }

    selected.dedup();

    match selected.as_slice() {
        [] => Ok(None),
        [backend] => Ok(Some(*backend)),
        _ => Err(CmdError::Other(
            "cannot combine multiple backend override flags".to_string(),
        )),
    }
}

fn exit_on_err(ace: &mut Ace, result: Result<(), CmdError>) {
    if let Err(e) = result {
        ace.error(&e.to_string());
        std::process::exit(1);
    }
}
