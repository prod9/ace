use std::io::IsTerminal;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

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

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub struct Io {
    mode: OutputMode,
    spinner: Option<ProgressBar>,
}

#[allow(dead_code)]
pub const LOGO: &str = r"
░█▀█░█▀▀░█▀▀
░█▀█░█░░░█▀▀
░▀░▀░▀▀▀░▀▀▀";

pub const LOGO_COLOR: &str = "\x1b[36m
░█▀█░█▀▀░█▀▀
░█▀█░█░░░█▀▀
░▀░▀░▀▀▀░▀▀▀\x1b[0m";

pub fn logo(mode: OutputMode) -> &'static str {
    match mode {
        OutputMode::Human => LOGO_COLOR,
        _ => "",
    }
}

impl Io {
    pub fn new(mode: OutputMode) -> Self {
        Self { mode, spinner: None }
    }

    // -- output --

    pub fn progress(&mut self, msg: &str) {
        self.clear_spinner();
        if self.mode != OutputMode::Human {
            return;
        }
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} {msg}")
                .expect("valid template"),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        self.spinner = Some(pb);
    }

    pub fn done(&mut self, msg: &str) {
        self.clear_spinner();
        match self.mode {
            OutputMode::Human => eprintln!("{} {msg}", style("✓").green()),
            OutputMode::Porcelain => eprintln!("{msg}"),
            OutputMode::Silent => {}
        }
    }

    pub fn warn(&mut self, msg: &str) {
        self.clear_spinner();
        match self.mode {
            OutputMode::Human => eprintln!("{} {msg}", style("⚠").yellow()),
            OutputMode::Porcelain => eprintln!("{msg}"),
            OutputMode::Silent => {}
        }
    }

    pub fn error(&mut self, msg: &str) {
        self.clear_spinner();
        match self.mode {
            OutputMode::Human => eprintln!("{} {msg}", style("✗").red()),
            OutputMode::Porcelain => eprintln!("{msg}"),
            OutputMode::Silent => {}
        }
    }

    pub fn hint(&mut self, msg: &str) {
        self.clear_spinner();
        match self.mode {
            OutputMode::Human => eprintln!("  {} {msg}", style("→").dim()),
            OutputMode::Porcelain => eprintln!("hint: {msg}"),
            OutputMode::Silent => {}
        }
    }

    pub fn data(&mut self, msg: &str) {
        self.clear_spinner();
        if self.mode == OutputMode::Silent {
            return;
        }
        println!("{msg}");
    }

    pub fn separator(&mut self) {
        self.clear_spinner();
        if self.mode == OutputMode::Human {
            eprintln!("\n{}\n", style("⟢⟢⟢⟢⟢⟢⟢").dim());
        }
    }

    // -- input --

    pub fn prompt_text(&mut self, prompt: &str, initial: Option<&str>) -> Result<String, IoError> {
        self.clear_spinner();
        let mut p = inquire::Text::new(prompt);
        if let Some(val) = initial {
            p = p.with_initial_value(val);
        }
        p.prompt().map_err(map_inquire_err)
    }

    pub fn prompt_select(&mut self, prompt: &str, options: Vec<String>) -> Result<String, IoError> {
        self.clear_spinner();
        inquire::Select::new(prompt, options)
            .prompt()
            .map_err(map_inquire_err)
    }

    fn clear_spinner(&mut self) {
        if let Some(sp) = self.spinner.take() {
            sp.finish_and_clear();
        }
    }
}

fn map_inquire_err(e: inquire::InquireError) -> IoError {
    match e {
        inquire::InquireError::OperationCanceled
        | inquire::InquireError::OperationInterrupted => IoError::Cancelled,
        other => IoError::Io(std::io::Error::other(other)),
    }
}
