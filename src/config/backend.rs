use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[default]
    Claude,
    OpenCode,
}

impl Backend {
    pub fn binary(&self) -> &'static str {
        match self {
            Backend::Claude => "claude",
            Backend::OpenCode => "opencode",
        }
    }

    pub fn skills_dir(&self) -> &'static str {
        match self {
            Backend::Claude => ".claude",
            Backend::OpenCode => ".opencode",
        }
    }

    pub fn instructions_file(&self) -> &'static str {
        match self {
            Backend::Claude => "CLAUDE.md",
            Backend::OpenCode => "AGENTS.md",
        }
    }
}
