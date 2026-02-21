pub mod authenticate;
pub mod clone_school;
pub mod install;
pub mod link;
pub mod write_config;

use std::path::Path;

pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}
