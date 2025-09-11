// This is the entry point of our `replay` CLI tool.
// It delegates execution to the `replay` crate, which handles
// Argument parsing, command dispatching and core logic.

use crossterm::terminal;
use replay_pty::run;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Err(err) = run(&args) {
        if let Err(e) = terminal::disable_raw_mode() {
            eprintln!("Impossible to quit the raw mode: {e}")
        }
        eprintln!("{}", err);
        terminal::disable_raw_mode().unwrap();
        process::exit(1)
    };
}
