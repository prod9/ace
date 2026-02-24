use std::path::Path;

use crate::config;

use super::CmdError;

pub fn run() {
    if let Err(e) = run_inner() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_inner() -> Result<(), CmdError> {
    let cwd = std::env::current_dir()?;
    let mut formatted = 0;

    let ace_path = cwd.join("ace.toml");
    if ace_path.exists() {
        format_ace_toml(&ace_path)?;
        formatted += 1;
    }

    let school_path = cwd.join("school.toml");
    if school_path.exists() {
        format_school_toml(&school_path)?;
        formatted += 1;
    }

    if formatted == 0 {
        eprintln!("no ace.toml or school.toml in current directory");
    }

    Ok(())
}

fn format_ace_toml(path: &Path) -> Result<(), CmdError> {
    let toml = config::ace_toml::load(path)?;
    config::ace_toml::save(path, &toml)?;
    eprintln!("Formatted {}", path.display());
    Ok(())
}

fn format_school_toml(path: &Path) -> Result<(), CmdError> {
    let toml = config::school_toml::load(path)?;
    config::school_toml::save(path, &toml)?;
    eprintln!("Formatted {}", path.display());
    Ok(())
}
