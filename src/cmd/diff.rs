use crate::ace::Ace;
use crate::config::ConfigError;

use super::CmdError;

pub async fn run(ace: &mut Ace) {
    let result = run_inner(ace);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    let cache = ace.require_school()?.cache.clone().ok_or(ConfigError::NoSchool)?;

    ace.data(&format!("# school-cache\t{}", cache.display()));

    let git = ace.git(&cache);
    git.intent_to_add_all()?;
    let out = git.diff()?;
    if !out.is_empty() {
        ace.data(&out);
    }

    Ok(())
}
