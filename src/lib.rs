//! # replay
//!
//! `replay` is the library for the `replay` CLI tool, a utility to
//! record, replay, and manage sequences of shell commands.
//!
//! This crate exposes the logic behind the CLI interface and is meant
//! to be used by the `replay` binary.
//!
//! ## Modules
//!
//! - [`args`] Defines the command-line interface using `clap`.
//! - [`commands`] Contains implementations of all supported subcommands.
//! - [`errors`] Defines custom error types for the library.

pub mod args;
pub mod char_buffer;
pub mod commands;
pub mod errors;
pub mod paths;
pub mod pty;
pub mod session;

use errors::Result;

use crate::args::parse_command;

/// Entrypoint called by the binary.
/// Parses CLI arguments and run the appropriate command.
pub fn run(args: &[String]) -> Result<()> {
    let cmd = parse_command(args)?;
    cmd.run()
}
