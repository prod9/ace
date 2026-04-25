use std::collections::HashMap;

use clap::Subcommand;

use crate::ace::Ace;
use crate::actions::project::edit_skills_config::{EditSkillsConfig, Op};
use crate::actions::project::list_skills::{build_rows, render_names, render_table};
use crate::config::Scope;
use crate::glob;
use crate::state::discover::{Tier, discover_skills};
use crate::state::resolver::resolve;

use super::CmdError;

#[derive(Subcommand)]
pub enum Command {
    /// Append patterns to include_skills (always-add)
    Include {
        /// One or more skill names or glob patterns
        #[arg(required = true)]
        patterns: Vec<String>,
    },
    /// Append patterns to exclude_skills (always-remove)
    Exclude {
        /// One or more skill names or glob patterns
        #[arg(required = true)]
        patterns: Vec<String>,
    },
    /// Drop entries from include_skills and/or exclude_skills
    Clear {
        /// Drop only include_skills
        #[arg(long)]
        include: bool,
        /// Drop only exclude_skills
        #[arg(long)]
        exclude: bool,
    },
}

pub fn run(ace: &mut Ace, command: Option<Command>, show_all: bool, names_only: bool) {
    let result = run_inner(ace, command, show_all, names_only);
    super::exit_on_err(ace, result);
}

fn run_inner(
    ace: &mut Ace,
    command: Option<Command>,
    show_all: bool,
    names_only: bool,
) -> Result<(), CmdError> {
    match command {
        None => list(ace, show_all, names_only),
        Some(Command::Include { patterns }) => mutate(ace, validate_all(&patterns)?, Op::Include),
        Some(Command::Exclude { patterns }) => mutate(ace, validate_all(&patterns)?, Op::Exclude),
        Some(Command::Clear { include, exclude }) => {
            mutate_op(ace, Op::Clear { include, exclude })
        }
    }
}

fn list(ace: &mut Ace, show_all: bool, names_only: bool) -> Result<(), CmdError> {
    ace.require_state()?;
    let (resolution, tiers) = collect(ace)?;
    let rows = build_rows(&resolution, &tiers, show_all);

    let output = if names_only {
        render_names(&rows)
    } else {
        render_table(&rows)
    };
    ace.data(&output);
    Ok(())
}

fn collect(ace: &mut Ace) -> Result<(crate::state::resolver::Resolution, HashMap<String, Tier>), CmdError> {
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
    Ok((resolution, tiers))
}

fn mutate<F>(ace: &mut Ace, patterns: Vec<String>, build_op: F) -> Result<(), CmdError>
where
    F: FnOnce(Vec<String>) -> Op,
{
    mutate_op(ace, build_op(patterns))
}

fn mutate_op(ace: &mut Ace, op: Op) -> Result<(), CmdError> {
    let scope = ace.scope_override().unwrap_or(Scope::Project);
    let summary = describe(&op);
    EditSkillsConfig { scope, op }.run(ace)?;
    ace.done(&format!("{summary} ({})", scope_name(scope)));
    Ok(())
}

fn validate_all(patterns: &[String]) -> Result<Vec<String>, CmdError> {
    for p in patterns {
        glob::validate(p).map_err(|e| CmdError::Other(format!("invalid pattern `{p}`: {e}")))?;
    }
    Ok(patterns.to_vec())
}

fn describe(op: &Op) -> String {
    match op {
        Op::Include(p) => format!("included {}", p.join(", ")),
        Op::Exclude(p) => format!("excluded {}", p.join(", ")),
        Op::Clear { include: true, exclude: false } => "cleared include_skills".to_string(),
        Op::Clear { include: false, exclude: true } => "cleared exclude_skills".to_string(),
        Op::Clear { .. } => "cleared include_skills and exclude_skills".to_string(),
    }
}

fn scope_name(s: Scope) -> &'static str {
    match s {
        Scope::User => "user",
        Scope::Project => "project",
        Scope::Local => "local",
    }
}
