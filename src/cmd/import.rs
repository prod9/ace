use crate::ace::Ace;
use crate::state::actions::import_skill::{ImportError, ImportResult, ImportSkill};

use super::CmdError;

pub fn run(ace: &mut Ace, source: &str, skill: Option<&str>) {
    let result = run_inner(ace, source, skill);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace, source: &str, skill: Option<&str>) -> Result<(), CmdError> {
    let school_root = ace.require_school()?.root.clone();

    let result = ImportSkill {
        source,
        skill,
        school_root: &school_root,
    }
    .run(ace)?;

    match result {
        ImportResult::Done { .. } => {}
        ImportResult::NeedsSelection(skills) => {
            let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
            let selected = inquire::Select::new("Multiple skills found, pick one:", names)
                .prompt()
                .map_err(|e| ImportError::Clone(format!("prompt: {e}")))?;

            let skill = skills.iter().find(|s| s.name == selected)
                .ok_or_else(|| ImportError::SkillNotFound(selected.to_string()))?;

            ImportSkill {
                source,
                skill: Some(&skill.name),
                school_root: &school_root,
            }
            .install_selected(skill, ace)?;
        }
    }
    Ok(())
}
