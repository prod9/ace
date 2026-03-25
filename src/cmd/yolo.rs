use crate::ace::Ace;
use crate::config::{ace_toml, paths};

use super::CmdError;

pub fn run(ace: &mut Ace) {
    let result = run_inner(ace);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    let ace_paths = paths::resolve(ace.project_dir())?;

    let mut local = ace_toml::load(&ace_paths.local).unwrap_or_default();
    local.yolo = true;
    ace_toml::save(&ace_paths.local, &local)?;

    ace.done("yolo mode enabled");
    Ok(())
}
