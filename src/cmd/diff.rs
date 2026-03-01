use crate::ace::Ace;
use crate::config::school_paths;

use super::CmdError;

pub async fn run(ace: &mut Ace) {
    let result = run_inner();
    super::exit_on_err(ace, result);
}

fn run_inner() -> Result<(), CmdError> {
    let cwd = std::env::current_dir()?;
    let mut ace = Ace::load(&cwd, Ace::term_sink())?;

    let spec = ace.state.school_specifier.as_deref()
        .ok_or(CmdError::NoSchool)?;

    let sp = school_paths::resolve(&cwd, spec)?;
    let cache = sp.cache.ok_or(CmdError::NoSchool)?;

    let out = crate::git::diff(&cache)?;
    if !out.is_empty() {
        ace.data(&out);
    }

    Ok(())
}
