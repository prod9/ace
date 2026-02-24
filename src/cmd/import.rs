use std::path::PathBuf;

use crate::state::actions::import_skill::{ImportError, ImportResult, ImportSkill};

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("{0}")]
    Import(#[from] ImportError),
    #[error("{0}")]
    Resolve(#[from] crate::config::school_paths::ResolveError),
    #[error("no school found, run ace setup or cd into a school repo")]
    NoSchool,
}

pub fn run(source: &str, skill: Option<&str>) {
    if let Err(e) = run_inner(source, skill) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_inner(source: &str, skill: Option<&str>) -> Result<(), RunError> {
    let school_root = resolve_school_root()?;

    let sp = crate::status::spinner(&format!("Importing from {source}"));
    let result = ImportSkill {
        source,
        skill,
        school_root: &school_root,
    }
    .run()?;
    sp.finish_and_clear();

    match result {
        ImportResult::Done { skill } => {
            crate::status::done(&format!("Imported skill: {skill}"));
        }
        ImportResult::NeedsSelection(skills) => {
            let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
            let selected = inquire::Select::new("Multiple skills found, pick one:", names)
                .prompt()
                .map_err(|e| ImportError::Clone(format!("prompt: {e}")))?;

            let skill = skills.iter().find(|s| s.name == selected)
                .ok_or_else(|| ImportError::SkillNotFound(selected.to_string()))?;

            let sp = crate::status::spinner(&format!("Installing {}", skill.name));
            ImportSkill {
                source,
                skill: Some(&skill.name),
                school_root: &school_root,
            }
            .install_selected(skill)?;
            sp.finish_and_clear();

            crate::status::done(&format!("Imported skill: {}", skill.name));
        }
    }
    Ok(())
}

fn resolve_school_root() -> Result<PathBuf, RunError> {
    let cwd = std::env::current_dir()?;

    // School repo context: school.toml in cwd
    if cwd.join("school.toml").exists() {
        return Ok(cwd);
    }

    // App repo context: resolve from ace.toml
    let ace_toml_path = cwd.join("ace.toml");
    if ace_toml_path.exists() {
        let ace = crate::config::ace_toml::load(&ace_toml_path)?;
        let paths = crate::config::school_paths::resolve(&cwd, &ace.school)?;
        return Ok(paths.root);
    }

    Err(RunError::NoSchool)
}
