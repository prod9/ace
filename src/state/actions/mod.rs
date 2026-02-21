pub mod authenticate;
pub mod download_school;
pub mod exec;
pub mod install;
pub mod link;
pub mod school_init;
pub mod setup;
pub mod sync_prompt;
pub mod write_config;

use std::path::Path;

pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}
