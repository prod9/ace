use crate::ace::Ace;
use crate::config::school_paths;

use super::CmdError;

pub async fn run(ace: &mut Ace) {
    let result = run_inner(ace);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    let cwd = std::env::current_dir()?;
    let mode = ace.output_mode();
    let mut ace = Ace::load(&cwd, mode)?;

    let spec = ace.state.school_specifier.as_deref()
        .ok_or(CmdError::NoSchool)?;

    let sp = school_paths::resolve(&cwd, spec)?;
    let cache = sp.cache.ok_or(CmdError::NoSchool)?;

    ace.data(&format!("# school-cache\t{}", cache.display()));

    let out = ace.git(&cache).diff()?;
    if !out.is_empty() {
        ace.data(&out);
    }

    Ok(())
}
