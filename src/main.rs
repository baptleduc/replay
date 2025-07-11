// This is the entry point of our `replay` CLI tool.
// It delegates execution to the `replay` crate, which handles
// Argument parsing, command dispatching and core logic.

use replay::run;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error during core application: {}", err);
        process::exit(1)
    };
}
