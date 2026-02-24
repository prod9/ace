use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::events::{Event, EventSink};

pub struct TermSink {
    spinner: Option<ProgressBar>,
}

impl TermSink {
    pub fn new() -> Self {
        Self { spinner: None }
    }
}

impl EventSink for TermSink {
    fn handle(&mut self, event: Event) {
        if let Some(sp) = self.spinner.take() {
            sp.finish_and_clear();
        }

        match event {
            Event::Progress(msg) => {
                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner} {msg}")
                        .expect("valid template"),
                );
                pb.set_message(msg);
                pb.enable_steady_tick(std::time::Duration::from_millis(80));
                self.spinner = Some(pb);
            }
            Event::Done(msg) => {
                eprintln!("{} {msg}", style("✓").green());
            }
            Event::Warn(msg) => {
                eprintln!("{} {msg}", style("⚠").yellow());
            }
            Event::Error(msg) => {
                eprintln!("{} {msg}", style("✗").red());
            }
            Event::Data(msg) => {
                println!("{msg}");
            }
        }
    }

    fn finish(&mut self) {
        if let Some(sp) = self.spinner.take() {
            sp.finish_and_clear();
        }
    }
}
