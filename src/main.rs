#![deny(warnings)]

mod ace;
mod actions;
mod cmd;
mod config;
mod fsutil;
mod git;
mod glob;
mod paths;
mod platform;
mod templates;
mod state;
mod upgrade;

use clap::Parser;
use cmd::Cli;
use ace::OutputMode;

fn main() {
    let cli = Cli::parse();
    let mode = OutputMode::detect(cli.porcelain);

    let logo = ace::logo(mode);
    if !logo.is_empty() {
        eprintln!("{logo}");
        eprintln!("  {}\n", env!("ACE_GIT_HASH"));
    }

    let project_dir = std::env::current_dir().expect("cannot determine current directory");
    let mut ace = ace::Ace::new(project_dir, mode);
    warn_stray_cache_dirs(&mut ace);
    smol::block_on(cmd::run(&mut ace, cli));
}

/// Startup hint: if the old flat cache layout (`~/.cache/ace/{owner/repo}/`)
/// has stray entries, nudge the user to clean them up. Self-silences once the user
/// deletes the strays. New layout: `~/.local/share/ace/` (schools) +
/// `~/.cache/ace/imports/` (upstream source snapshots).
fn warn_stray_cache_dirs(ace: &mut ace::Ace) {
    let Ok(cache_root) = config::paths::ace_cache_dir() else {
        return;
    };

    let stray = config::paths::detect_stray_cache_dirs(&cache_root);
    if stray.is_empty() {
        return;
    }

    ace.warn(&format!(
        "old ACE cache layout detected at {} ({} stray entr{}); safe to delete",
        cache_root.display(),
        stray.len(),
        if stray.len() == 1 { "y" } else { "ies" },
    ));
}
