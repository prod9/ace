mod ace;
mod cmd;
mod config;
mod events;
mod git;
mod templates;
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

    let project_dir = std::env::current_dir().expect("cannot determine current directory");
    let mut ace = ace::Ace::new(project_dir, mode);
    smol::block_on(cmd::run(&mut ace, cli));
}
