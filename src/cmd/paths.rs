use crate::ace::Ace;
use crate::config::{paths, school_paths};

use super::CmdError;

pub async fn run(ace: &mut Ace) {
    let result = run_inner(ace);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    let cwd = std::env::current_dir()?;
    let p = paths::resolve(&cwd)?;

    ace.data(&format!("config.user\t{}", p.user.display()));
    ace.data(&format!("config.local\t{}", p.local.display()));
    ace.data(&format!("config.project\t{}", p.project.display()));

    if let Some(spec) = ace.state.school_specifier.as_deref() {
        let sp = school_paths::resolve(&cwd, spec)?;

        ace.data(&format!("school.source\t{}", sp.source));
        if let Some(ref path) = sp.cache {
            ace.data(&format!("school.cache\t{}", path.display()));
        }
        ace.data(&format!("school.root\t{}", sp.root.display()));
    }

    Ok(())
}
