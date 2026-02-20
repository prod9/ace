mod ace;
mod cmd;
mod config;
mod session;
mod ui;

use std::collections::HashMap;

use clap::Parser;
use cmd::Cli;
use ui::StdoutUI;

fn main() {
    let cli = Cli::parse();
    let config = config::Config {
        school_specifier: None,
        schools: HashMap::new(),
    };
    let ui = Box::new(StdoutUI);
    let ace = ace::Ace::new(config, ui);

    smol::block_on(cmd::run(&ace, cli));
}
