mod ace;
mod cmd;
mod config;
mod git;
mod templates;
mod state;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::Parser;
use cmd::Cli;
use ace::OutputMode;

fn main() {
    let cli = Cli::parse();
    let mode = OutputMode::detect(cli.porcelain);

    install_signal_handler(mode);

    let logo = ace::logo(mode);
    if !logo.is_empty() {
        eprintln!("{logo}");
        eprintln!("  {}\n", env!("ACE_GIT_HASH"));
    }

    let project_dir = std::env::current_dir().expect("cannot determine current directory");
    let mut ace = ace::Ace::new(project_dir, mode);
    smol::block_on(cmd::run(&mut ace, cli));
}

/// Restore terminal state on Ctrl+C before exit.
///
/// During ACE's TUI phases (spinners, prompts), indicatif or inquire may
/// hide the cursor or change terminal state. This handler ensures the
/// cursor is restored and the process exits cleanly on SIGINT.
fn install_signal_handler(mode: OutputMode) {
    if mode != OutputMode::Human {
        return;
    }

    let interrupted = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&interrupted);

    // Register a non-blocking flag — signal_hook handles the signal safely.
    signal_hook::flag::register(signal_hook::consts::SIGINT, flag).ok();

    std::thread::spawn(move || {
        loop {
            if interrupted.load(Ordering::Relaxed) {
                // Restore cursor visibility and exit.
                eprint!("\x1b[?25h");
                std::process::exit(130); // 128 + SIGINT (2)
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });
}
