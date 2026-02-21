use clap::Subcommand;

use crate::ace::Ace;
use crate::state::actions::school_init::SchoolInit;

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new school repository
    Init {
        /// School display name
        #[arg(long)]
        name: Option<String>,
    },
}

pub async fn run(ace: &mut Ace, command: Command) {
    match command {
        Command::Init { name } => {
            let project_dir = match std::env::current_dir() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            };

            let init = SchoolInit {
                name: name.as_deref(),
                project_dir: &project_dir,
            };

            let mut session = ace.session();
            if let Err(e) = init.run(&mut session).await {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
