use clap::Subcommand;

use crate::ace::Ace;
use crate::state::actions::propose::Propose;
use crate::state::actions::school_init::SchoolInit;
use crate::term_ui::{Screen, Tui};

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new school repository
    Init {
        /// School display name
        #[arg(long)]
        name: Option<String>,
    },
    /// Propose local school changes back to upstream
    Propose,
}

pub async fn run(ace: &mut Ace, command: Command) {
    match command {
        Command::Init { name } => run_init(ace, name),
        Command::Propose => run_propose(ace),
    }
}

fn run_init(ace: &mut Ace, name: Option<String>) {
    match name {
        Some(name) => {
            let project_dir = match std::env::current_dir() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            };

            let init = SchoolInit {
                name: &name,
                project_dir: &project_dir,
            };

            let mut session = ace.session();
            if let Err(e) = init.run(&mut session) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        None => {
            if let Err(e) = Tui::new(ace).show(Screen::SchoolInit) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn run_propose(_ace: &mut Ace) {
    let project_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    // Need to load state to know which school is linked
    let state = match crate::state::State::load(&project_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let mut ace_with_state = crate::ace::Ace::with_state(state);
    let mut session = ace_with_state.session();

    let propose = Propose {
        project_dir: &project_dir,
    };

    match propose.run(&mut session) {
        Ok(url) => println!("PR created: {url}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
