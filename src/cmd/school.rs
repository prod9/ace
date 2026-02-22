use clap::Subcommand;

use crate::ace::Ace;
use crate::state::actions::school_propose::SchoolPropose;
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
    #[clap(alias = "pr")]
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

    let state = match crate::state::State::load(&project_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let specifier = match &state.school_specifier {
        Some(s) => s.clone(),
        None => {
            eprintln!("error: no school linked, run ace setup first");
            std::process::exit(1);
        }
    };

    let repo_key = specifier.split_once(':').map_or(specifier.as_str(), |(repo, _)| repo);
    let token = match load_github_token(repo_key) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let mut ace_with_state = crate::ace::Ace::with_state(state);
    let mut session = ace_with_state.session();

    let propose = SchoolPropose {
        project_dir: &project_dir,
        token: &token,
    };

    match propose.run(&mut session) {
        Ok(url) => println!("PR created: {url}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn load_github_token(repo_key: &str) -> Result<String, String> {
    let path = crate::config::user_config::default_path()
        .ok_or("cannot determine config dir")?;
    let config = crate::config::user_config::load(&path)
        .map_err(|e| format!("load config: {e}"))?;
    let school = config
        .get(repo_key)
        .ok_or(format!("no config for school {repo_key}, run ace setup"))?;
    let github = school
        .services
        .get("github")
        .ok_or(format!("no github token for {repo_key}, run ace auth"))?;
    github
        .token
        .clone()
        .ok_or(format!("github token empty for {repo_key}"))
}
