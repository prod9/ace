use crate::ace::Ace;
use crate::config::{paths, school_paths};

pub async fn run(ace: &mut Ace) {
    let session = ace.session();
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let p = match paths::resolve(&cwd) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    println!("config.user\t{}", p.user.display());
    println!("config.local\t{}", p.local.display());
    println!("config.project\t{}", p.project.display());

    let specifier = session.state.school_specifier.as_deref();
    if let Some(spec) = specifier {
        let sp = match school_paths::resolve(&cwd, spec) {
            Ok(sp) => sp,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        };

        println!("school.source\t{}", sp.source);
        if let Some(ref path) = sp.cache {
            println!("school.cache\t{}", path.display());
        }
        println!("school.root\t{}", sp.root.display());
    }
}
