use crate::ace::Ace;
use crate::config;
use crate::state::State;
use crate::state::actions::exec::{self, Exec};
use crate::state::actions::prepare::Prepare;
use crate::state::actions::sync_prompt::SyncPrompt;

pub async fn run(ace: &mut Ace) {
    let project_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    match State::load(&project_dir) {
        Ok(state) => {
            *ace.state_mut() = state;
        }
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("hint: run `ace setup <owner/repo>` first");
            std::process::exit(1);
        }
    }

    let mut session = ace.session();

    let specifier = match &session.state.school_specifier {
        Some(s) => s.clone(),
        None => {
            eprintln!("error: no school configured, run `ace setup`");
            std::process::exit(1);
        }
    };

    if let Err(e) = (Prepare {
        specifier: &specifier,
        project_dir: &project_dir,
    })
    .run(&mut session)
    .await
    {
        eprintln!("error: {e}");
        std::process::exit(1);
    }

    let school_paths = match config::school_paths::resolve(&project_dir, &specifier) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let school_toml_path = school_paths.root.join("school.toml");
    let school_toml = match config::school_toml::load(&school_toml_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: school.toml: {e}");
            std::process::exit(1);
        }
    };

    let system_prompt = {
        let sync = SyncPrompt {
            school_root: &school_paths.root,
            school_name: &school_toml.school.name,
            school_description: school_toml.school.description.as_deref(),
        };
        match sync.run(&mut session) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    };

    let backend = match exec::detect_backend() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let env = session.state.env.clone();
    let action = Exec {
        backend,
        system_prompt,
        project_dir: project_dir.clone(),
        env,
    };
    if let Err(e) = action.run(&mut session) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
