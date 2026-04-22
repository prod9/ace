pub mod clone;
pub mod link;
pub mod prepare;
pub mod pull;
pub mod register_mcp;
pub mod remove_mcp;
pub mod setup;
pub mod update_gitignore;

pub use link::{Link, SCHOOL_FOLDERS};
pub use prepare::{Prepare, PrepareError, PrepareResult};
pub use pull::{ChangeKind, Pull, PullOutcome, SkillChange};
pub use register_mcp::{RegisterMcp, RegisterMcpError};
pub use remove_mcp::RemoveMcp;
pub use setup::{Setup, SetupError};
pub use update_gitignore::UpdateGitignore;
