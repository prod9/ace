use crate::ace::Ace;
use crate::config;
use crate::state::State;
use crate::state::actions::download_school::DownloadSchool;
use crate::state::actions::exec::{self, Exec};
use crate::state::actions::sync_prompt::SyncPrompt;
use crate::state::actions::sync_skills::SyncSkills;

pub async fn run(ace: &mut Ace) {
    let project_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    // Load state from config files
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

    // Resolve school paths
    let school_paths = match config::school_paths::resolve(&project_dir, &specifier) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    // Fetch school (git pull if cached, clone if not)
    if let Err(e) = (DownloadSchool { paths: &school_paths }).run(&mut session) {
        eprintln!("warning: school fetch failed: {e}");
        // Continue — cached state may still work
    }

    // Load school.toml for metadata
    let school_toml_path = school_paths.root.join("school.toml");
    let school_toml = match config::school_toml::load(&school_toml_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: school.toml: {e}");
            std::process::exit(1);
        }
    };

    // Sync skills from school into project
    match (SyncSkills {
        school_root: &school_paths.root,
        project_dir: &project_dir,
    })
    .run(&mut session)
    {
        Ok(result) if result.synced > 0 => {
            eprintln!("Synced {} skills", result.synced);
        }
        Err(e) => {
            eprintln!("warning: skill sync failed: {e}");
        }
        _ => {}
    }

    // Build system prompt
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

    // Detect backend
    let backend = match exec::detect_backend() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    // Exec into backend (replaces this process)
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
