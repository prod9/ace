pub mod discover_skill;
pub mod import_skill;
pub mod init_school;
pub mod install_school;
pub mod link_school;
pub mod prepare_school;
pub mod register_mcp;
pub mod remove_mcp;
pub mod setup_project;
pub mod update_cache;
pub mod update_gitignore;
pub mod update_school;
pub mod write_config;

use std::path::Path;

pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}
