use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::events::OutputMode;

pub struct EventSink {
    mode: OutputMode,
    spinner: Option<ProgressBar>,
}

impl EventSink {
    pub fn new(mode: OutputMode) -> Self {
        Self { mode, spinner: None }
    }

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

    fn clear_spinner(&mut self) {
        if let Some(sp) = self.spinner.take() {
            sp.finish_and_clear();
        }
    }
}
