use clap::Subcommand;

use crate::ace::Ace;
use crate::actions::project::edit_skills_config::{EditSkillsConfig, Op};
use crate::actions::project::list_skills::{render_names, render_table};
use crate::config::Scope;
use crate::glob;
use crate::state::skills::Skills;

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
    /// Reset include_skills and/or exclude_skills back to empty
    Reset {
        /// Reset only include_skills
        #[arg(long)]
        include: bool,
        /// Reset only exclude_skills
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
        Some(Command::Reset { include, exclude }) => {
            mutate_op(ace, Op::Reset { include, exclude })
        }
    }
}

fn list(ace: &mut Ace, show_all: bool, names_only: bool) -> Result<(), CmdError> {
    ace.require_state()?;
    let school_root = ace.require_school()?.root.clone();
    let skills = Skills::discover(&school_root)?.resolve(&ace.state().config);

    let output = if names_only {
        render_names(&skills, show_all)
    } else {
        render_table(&skills, show_all)
    };
    ace.data(&output);
    Ok(())
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
        Op::Reset { include: true, exclude: false } => "reset include_skills".to_string(),
        Op::Reset { include: false, exclude: true } => "reset exclude_skills".to_string(),
        Op::Reset { .. } => "reset include_skills and exclude_skills".to_string(),
    }
}

fn scope_name(s: Scope) -> &'static str {
    match s {
        Scope::User => "user",
        Scope::Project => "project",
        Scope::Local => "local",
    }
}
