use crate::ace::Ace;
use crate::config::{paths, school_paths};

use super::CmdError;

pub async fn run(ace: &mut Ace) {
    if let Err(e) = run_inner(ace) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    let session = ace.session();
    let cwd = std::env::current_dir()?;
    let p = paths::resolve(&cwd)?;

    println!("config.user\t{}", p.user.display());
    println!("config.local\t{}", p.local.display());
    println!("config.project\t{}", p.project.display());

    if let Some(spec) = session.state.school_specifier.as_deref() {
        let sp = school_paths::resolve(&cwd, spec)?;

        println!("school.source\t{}", sp.source);
        if let Some(ref path) = sp.cache {
            println!("school.cache\t{}", path.display());
        }
        println!("school.root\t{}", sp.root.display());
    }

    Ok(())
}
