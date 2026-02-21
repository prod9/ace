use clap::Subcommand;

use crate::ace::Ace;

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new school repository
    Init,
}

pub async fn run(ace: &mut Ace, command: Command) {
    match command {
        Command::Init => ace.session().ui.message("school init").await,
    }
}
