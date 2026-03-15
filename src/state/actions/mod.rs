pub mod exec;
pub mod import_skill;
pub mod install;
pub mod link;
pub mod prepare;
pub mod register_mcp;
pub mod school_init;
pub mod school_update;
pub mod setup;
pub mod update;
pub mod utils;
pub mod write_config;

use std::path::Path;

pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}
