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

    let out = ace.git(&cache).diff()?;
    if !out.is_empty() {
        ace.data(&out);
    }

    Ok(())
}
