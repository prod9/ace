use std::path::PathBuf;

use crate::ace::Ace;
use crate::state::actions::import_skill::{ImportError, ImportResult, ImportSkill};

use super::CmdError;

pub fn run(ace: &mut Ace, source: &str, skill: Option<&str>) {
    let result = run_inner(source, skill);
    super::exit_on_err(ace, result);
}

fn run_inner(source: &str, skill: Option<&str>) -> Result<(), CmdError> {
    let school_root = resolve_school_root()?;

    let mut ace = crate::ace::Ace::new(crate::ace::Ace::term_sink());

    let result = ImportSkill {
        source,
        skill,
        school_root: &school_root,
    }
    .run(&mut ace)?;

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
            .install_selected(skill, &mut ace)?;
        }
    }
    Ok(())
}

fn resolve_school_root() -> Result<PathBuf, CmdError> {
    let cwd = std::env::current_dir()?;

    if cwd.join("school.toml").exists() {
        return Ok(cwd);
    }

    let ace_toml_path = cwd.join("ace.toml");
    if ace_toml_path.exists() {
        let ace = crate::config::ace_toml::load(&ace_toml_path)?;
        let paths = crate::config::school_paths::resolve(&cwd, &ace.school)?;
        return Ok(paths.root);
    }

    Err(CmdError::NoSchool)
}
