mod cmd;
mod config;
mod session;

use clap::Parser;
use cmd::Cli;

fn main() {
    let cli = Cli::parse();
    cmd::run(cli);
}
