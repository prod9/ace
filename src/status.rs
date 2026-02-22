use console::style;
use indicatif::{ProgressBar, ProgressStyle};

/// Print a status message: ✓ message
pub fn done(msg: &str) {
    eprintln!("{} {msg}", style("✓").green());
}

/// Create a spinner for long-running operations. Call .finish_and_clear() when done.
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .expect("valid template"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
