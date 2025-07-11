//! # replay
//!
//! `replay` is the core library for the `replay` CLI tool  a utility to
//! record, replay, and manage sequences of shell commands.
//!
//! This crate exposes the main logic behind the CLI interface and is meant
//! to be used by the `replay` binary.
//!
//! ## Modules
//!
//! - [`args`] Defines the command-line interface using `clap`.
//! - [`commands`] Contains implementations of all supported subcommands.
//!
//! ## Usage
//!
//! ```no_run
//! fn main() -> anyhow::Result<()> {
//!     replay::run()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Entry Point
//!
//! Use [`run`] to execute the CLI logic from a binary.

use std::error::Error;

pub mod args;
pub mod commands;

/// Entrypoint called by the binary.
/// Parses CLI arguments and dispatches the appropriate command.
pub fn run() -> Result<(), Box<dyn Error>> {
    Ok(())
}
