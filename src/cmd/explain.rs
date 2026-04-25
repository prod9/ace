use crate::ace::Ace;
use crate::actions::project::explain_skill::{ExplainError, find_or_suggest, render};
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

    match find_or_suggest(&skills, name) {
        Ok(skill) => {
            ace.data(&render(skill));
            Ok(())
        }
        Err(ExplainError::NotFound { name, near }) => {
            Err(CmdError::Other(format_not_found(&name, &near)))
        }
    }
}

fn format_not_found(name: &str, near: &[String]) -> String {
    if near.is_empty() {
        format!("unknown skill `{name}`")
    } else {
        format!("unknown skill `{name}` — did you mean: {}", near.join(", "))
    }
}
