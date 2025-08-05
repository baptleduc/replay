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
pub mod commands;
pub mod errors;
pub mod session;

use args::CliParser;
use clap::Parser;
use errors::ReplayError;

use crate::args::{CliCommand, parse_command};

/// Entrypoint called by the binary.
/// Parses CLI arguments and run the appropriate command.
pub fn run(args: &[String]) -> Result<(), ReplayError> {
    let cmd = parse_command(&args)?;
    cmd.run()
}
