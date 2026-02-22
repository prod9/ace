mod ace;
mod cmd;
mod config;
mod session;
mod state;
mod status;
mod term_ui;

use clap::Parser;
use cmd::Cli;

fn main() {
    let cli = Cli::parse();

    eprintln!("{}", term_ui::LOGO);
    eprintln!("  {}\n", env!("ACE_GIT_HASH"));

    let mut ace = ace::Ace::new();
    smol::block_on(cmd::run(&mut ace, cli));
}
