use std::io::IsTerminal;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    #[default]
    Human,
    Porcelain,
    Silent,
}

impl OutputMode {
    pub fn detect(porcelain: bool) -> Self {
        if porcelain || !std::io::stderr().is_terminal() {
            Self::Porcelain
        } else {
            Self::Human
        }
    }
}
