use crate::ace::Ace;
use crate::config::ConfigError;

use super::CmdError;

pub async fn run(ace: &mut Ace) {
    let result = run_inner(ace);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    let clone_path = ace.require_school()?.clone_path.clone().ok_or(ConfigError::NoSchool)?;

    ace.data(&format!("# school-clone\t{}", clone_path.display()));

    let git = ace.git(&clone_path);
    git.intent_to_add_all()?;
    let out = git.diff()?;
    if !out.is_empty() {
        ace.data(&out);
    }

    Ok(())
}
