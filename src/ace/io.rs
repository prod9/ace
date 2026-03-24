use std::io::{IsTerminal, Write as _};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;

// ANSI escape sequences for terminal state management.
// Alt screen is a separate buffer that preserves the user's scrollback.
// Cursor hide/show prevents a flickering cursor during full-screen redraws.
pub const ENTER_ALT_SCREEN: &[u8] = b"\x1b[?1049h"; // switch to alt screen buffer
pub const HIDE_CURSOR: &[u8] = b"\x1b[?25l";

/// Terminal cleanup sequence: leave alt screen (`\x1b[?1049l`) + show cursor (`\x1b[?25h`).
/// Safe to run even if alt screen was never entered (leave-alt-screen is a no-op).
const TERMINAL_CLEANUP: &[u8] = b"\x1b[?1049l\x1b[?25h";

/// RAII guard that restores terminal state on drop and on SIGINT.
///
/// Created once per session in Human mode. Spawns a background thread that
/// blocks on a Unix signal fd — no polling. On Ctrl+C the thread writes
/// cleanup escapes and exits with code 130 (128 + SIGINT).
struct TerminalGuard {
    handle: signal_hook::iterator::Handle,
}

impl TerminalGuard {
    fn new() -> Self {
        let mut signals = Signals::new(&[SIGINT]).expect("register signal handler");
        let handle = signals.handle();

        std::thread::spawn(move || {
            for _ in signals.forever() {
                let _ = std::io::stderr().write_all(TERMINAL_CLEANUP);
                let _ = std::io::stderr().flush();
                std::process::exit(130);
            }
        });

        Self { handle }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = std::io::stderr().write_all(TERMINAL_CLEANUP);
        let _ = std::io::stderr().flush();
        self.handle.close();
    }
}

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
    _guard: Option<TerminalGuard>,
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
        let guard = match mode {
            OutputMode::Human => Some(TerminalGuard::new()),
            _ => None,
        };
        Self { mode, spinner: None, _guard: guard }
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
