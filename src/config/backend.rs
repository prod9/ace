use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
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
}
