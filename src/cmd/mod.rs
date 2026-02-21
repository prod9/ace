mod auth;
mod config;
mod main;
mod paths;
mod school;
mod setup;

use clap::{Parser, Subcommand};

use crate::ace::Ace;

#[derive(Parser)]
#[command(name = "ace", about = "AI Coding Environment")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
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
    /// Print effective configuration
    Config,
    /// Print resolved filesystem paths ACE uses
    Paths,
    /// Manage schools
    School {
        #[command(subcommand)]
        command: school::Command,
    },
}

pub async fn run(ace: &mut Ace, cli: Cli) {
    match cli.command {
        Some(Command::Setup { specifier }) => setup::run(ace, specifier.as_deref()).await,
        Some(Command::Auth { name }) => auth::run(ace, &name).await,
        Some(Command::Config) => config::run(ace).await,
        Some(Command::Paths) => paths::run(ace).await,
        Some(Command::School { command }) => school::run(ace, command).await,
        None => main::run(ace).await,
    }
}
