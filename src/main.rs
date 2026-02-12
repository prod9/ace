mod ace;
mod cmd;
mod config;
mod session;
mod ui;

use std::collections::HashMap;

use clap::Parser;
use cmd::Cli;

fn main() {
    let cli = Cli::parse();
    let config = config::Config {
        schools: HashMap::new(),
    };
    let ui = Box::new(StdoutUI);
    let ace = ace::Ace::new(config, ui);

    smol::block_on(cmd::run(&ace, cli));
}

// Minimal UI impl until we build the real TUI.
struct StdoutUI;

impl ui::UI for StdoutUI {
    fn message(&self, text: &str) -> ui::UIFuture<'_, ()> {
        println!("{text}");
        Box::pin(std::future::ready(()))
    }

    fn confirm(&self, _prompt: &str) -> ui::UIFuture<'_, bool> {
        Box::pin(std::future::ready(false))
    }

    fn ask(&self, _prompt: &str) -> ui::UIFuture<'_, String> {
        Box::pin(std::future::ready(String::new()))
    }

    fn select(&self, _prompt: &str, _options: &[&str]) -> ui::UIFuture<'_, usize> {
        Box::pin(std::future::ready(0))
    }
}
