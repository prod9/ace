pub mod discover;
pub mod imports;
pub mod mcp;
pub mod prepare;
pub mod project;
pub mod school;

pub use discover::{DiscoveredSkill, Tier, discover_skills};
pub use prepare::{Prepare, PrepareError, PrepareResult};

use std::path::Path;

pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}
