mod auth;
mod config;
mod learn;
mod main;
mod setup;

use clap::{Parser, Subcommand};

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
}

pub fn run(cli: Cli) {
    match cli.command {
        Some(Command::Setup {
            school_name,
            source,
        }) => setup::run(&school_name, &source),
        Some(Command::Learn) => learn::run(),
        Some(Command::Auth { name }) => auth::run(&name),
        Some(Command::Config) => config::run(),
        None => main::run(),
    }
}
