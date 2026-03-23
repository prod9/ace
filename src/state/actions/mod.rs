pub mod discover_skill;
pub mod exec;
pub mod gitignore;
pub mod import_skill;
pub mod install;
pub mod link;
pub mod mcp_register;
pub mod prepare;
pub mod school_init;
pub mod school_update;
pub mod setup;
pub mod update;
pub mod write_config;

use std::path::Path;

pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}
