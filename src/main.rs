mod ace;
mod cmd;
mod config;
mod events;
mod git;
mod prompts;
mod session;
mod state;
mod term_ui;

use clap::Parser;
use cmd::Cli;

fn main() {
    let cli = Cli::parse();

    eprintln!("{}", term_ui::LOGO);
    eprintln!("  {}\n", env!("ACE_GIT_HASH"));

    let mut ace = ace::Ace::new(ace::Ace::term_sink());
    smol::block_on(cmd::run(&mut ace, cli));
}
