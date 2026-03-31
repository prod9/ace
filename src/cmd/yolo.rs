use crate::ace::Ace;
use crate::config::ace_toml::{self, Trust};
use crate::config::paths;

use super::CmdError;

pub fn run(ace: &mut Ace, trust: Trust) {
    let result = run_inner(ace, trust);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace, trust: Trust) -> Result<(), CmdError> {
    let ace_paths = paths::resolve(ace.project_dir())?;

    let mut local = ace_toml::load(&ace_paths.local).unwrap_or_default();
    local.trust = trust;
    local.yolo = false; // clear deprecated field
    ace_toml::save(&ace_paths.local, &local)?;

    let msg = match trust {
        Trust::Auto => "Auto mode — AI decides which actions need approval",
        Trust::Yolo => "Yolo mode — all permission prompts disabled",
        Trust::Default => "Default mode — using backend's standard permission handling",
    };
    ace.done(msg);
    Ok(())
}
