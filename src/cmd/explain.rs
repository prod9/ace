use crate::ace::Ace;
use crate::actions::project::explain_skill::{find_or_suggest, render};
use crate::state::skills::Skills;

use super::CmdError;

pub fn run(ace: &mut Ace, name: &str) {
    let result = run_inner(ace, name);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace, name: &str) -> Result<(), CmdError> {
    ace.require_state()?;
    let school_root = ace.require_school()?.root.clone();
    let skills = Skills::discover(&school_root)?.resolve(&ace.state().config);

    let skill = find_or_suggest(&skills, name).map_err(|e| CmdError::Other(e.to_string()))?;
    ace.data(&render(skill));
    Ok(())
}
