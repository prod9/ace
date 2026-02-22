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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_names() {
        assert_eq!(Backend::Claude.binary(), "claude");
        assert_eq!(Backend::OpenCode.binary(), "opencode");
    }

    #[test]
    fn skills_dirs() {
        assert_eq!(Backend::Claude.skills_dir(), ".claude");
        assert_eq!(Backend::OpenCode.skills_dir(), ".opencode");
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct Wrapper {
        backend: Backend,
    }

    #[test]
    fn deserialize_lowercase() {
        let w: Wrapper = toml::from_str("backend = \"claude\"")
            .expect("deserialize claude");
        assert_eq!(w.backend, Backend::Claude);

        let w: Wrapper = toml::from_str("backend = \"opencode\"")
            .expect("deserialize opencode");
        assert_eq!(w.backend, Backend::OpenCode);
    }

    #[test]
    fn serialize_lowercase() {
        let w = Wrapper { backend: Backend::Claude };
        let s = toml::to_string(&w).expect("serialize claude");
        assert!(s.contains("claude"));

        let w = Wrapper { backend: Backend::OpenCode };
        let s = toml::to_string(&w).expect("serialize opencode");
        assert!(s.contains("opencode"));
    }
}
