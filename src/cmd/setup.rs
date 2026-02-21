use crate::ace::Ace;
use crate::state::setup::Setup;

pub async fn run(ace: &mut Ace, specifier: Option<&str>) {
    let project_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let setup = Setup {
        specifier,
        project_dir: &project_dir,
    };

    let mut session = ace.session();
    match setup.run(&mut session).await {
        Ok(()) => session.ui.message("Setup complete.").await,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
