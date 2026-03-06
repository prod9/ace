use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use console::Term;

use crate::events::OutputMode;

const PLANE: &str = r#"  ✈ "#;
const FRAME_MS: u64 = 30;

pub fn run(mode: OutputMode) {
    if mode != OutputMode::Human {
        return;
    }

    let term = Term::stderr();
    let width = term.size().1 as usize;
    let trail_chars = ['·', '·', '·', '-', '-', '='];

    for pos in 0..width + PLANE.len() {
        let pad = pos.saturating_sub(PLANE.len());
        let visible_plane = if pos < PLANE.len() {
            &PLANE[PLANE.len() - pos..]
        } else {
            PLANE
        };

        let trail_len = pad.min(trail_chars.len());
        let trail: String = trail_chars[trail_chars.len() - trail_len..]
            .iter()
            .collect();

        let gap = pad.saturating_sub(trail_len);
        let line = format!("\r{:gap$}{trail}{visible_plane}", "");

        // Truncate to terminal width
        let display: String = line.chars().take(width + 1).collect();
        eprint!("{display}");
        let _ = io::stderr().flush();
        thread::sleep(Duration::from_millis(FRAME_MS));
    }

    // Clear the line and print landing message
    eprint!("\r{:width$}\r", "");
    let _ = io::stderr().flush();
}
