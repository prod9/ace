use std::collections::HashMap;

use crate::ace::Ace;
use crate::actions::project::explain_skill::{ExplainError, ExplainOutput, Status, build};
use crate::state::discover::{Tier, discover_skills};
use crate::state::resolver::resolve;

use super::CmdError;

pub fn run(ace: &mut Ace, name: &str) {
    let result = run_inner(ace, name);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace, name: &str) -> Result<(), CmdError> {
    ace.require_state()?;
    let school_root = ace.require_school()?.root.clone();
    let state = ace.state();
    let tree = &state.config;

    let discovered = discover_skills(&school_root)?;
    let names: Vec<String> = discovered.iter().map(|d| d.name.clone()).collect();
    let tiers: HashMap<String, Tier> = discovered
        .iter()
        .map(|d| (d.name.clone(), d.tier))
        .collect();

    let resolution = resolve(&names, &tree.ace_user, &tree.ace_project, &tree.ace_local);

    match build(&resolution, &tiers, name) {
        Ok(out) => {
            ace.data(&render(&out));
            Ok(())
        }
        Err(ExplainError::NotFound { name, near }) => Err(CmdError::Other(format_not_found(&name, &near))),
    }
}

fn render(out: &ExplainOutput) -> String {
    let tier = match out.tier {
        Some(Tier::Curated) => "curated",
        Some(Tier::Experimental) => "experimental",
        Some(Tier::System) => "system",
        None => "-",
    };
    let status = match out.status {
        Status::Active => "active",
        Status::Excluded => "excluded",
    };

    let mut s = format!("{} ({tier})\n  status: {status}\n  trace:\n", out.name);
    for line in &out.trace_lines {
        s.push_str("    ");
        s.push_str(line);
        s.push('\n');
    }
    s
}

fn format_not_found(name: &str, near: &[String]) -> String {
    if near.is_empty() {
        format!("unknown skill `{name}`")
    } else {
        format!("unknown skill `{name}` — did you mean: {}", near.join(", "))
    }
}
