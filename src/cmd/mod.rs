mod auth;
mod config;
mod learn;
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
    /// Set up a new school
    Setup {
        /// Name for this school (e.g. prodigy9, acme)
        school_name: String,
        /// Git-cloneable URL or local path to the school repository
        source: String,
    },
    /// Open the school in an AI coding tool for learning
    Learn,
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

pub async fn run(ace: &Ace, cli: Cli) {
    match cli.command {
        Some(Command::Setup {
            school_name,
            source,
        }) => setup::run(ace, &school_name, &source).await,
        Some(Command::Learn) => learn::run(ace).await,
        Some(Command::Auth { name }) => auth::run(ace, &name).await,
        Some(Command::Config) => config::run(ace).await,
        Some(Command::Paths) => paths::run(ace).await,
        Some(Command::School { command }) => school::run(ace, command).await,
        None => main::run(ace).await,
    }
}
