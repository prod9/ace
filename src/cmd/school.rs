use clap::Subcommand;

use crate::ace::Ace;

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new school repository
    Init,
}

pub async fn run(ace: &Ace, command: Command) {
    match command {
        Command::Init => ace.ui().message("school init").await,
    }
}
