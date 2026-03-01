mod ace;
mod cmd;
mod config;
mod events;
mod git;
mod prompts;
mod state;
mod term_ui;

use clap::Parser;
use cmd::Cli;
use events::OutputMode;

fn main() {
    let cli = Cli::parse();
    let mode = OutputMode::detect(cli.porcelain);

    let logo = term_ui::logo(mode);
    if !logo.is_empty() {
        eprintln!("{logo}");
        eprintln!("  {}\n", env!("ACE_GIT_HASH"));
    }

    let mut ace = ace::Ace::new(mode);
    smol::block_on(cmd::run(&mut ace, cli));
}
