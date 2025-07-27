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
//!
//! ## Entry Point
//!
//! Use [`run`] to execute the CLI logic from a binary.

pub mod args;
pub mod commands;
pub mod errors;

use args::CliParser;
use clap::Parser;
use errors::ReplayError;

use crate::args::{CliCommand, parse_command};

static DEFAULT_SESSION_PATH: &str = "~/.replay/session.json";

/// Entrypoint called by the binary.
/// Parses CLI arguments and run the appropriate command.
pub fn run(args: &[String]) -> Result<(), ReplayError> {
    let cmd = parse_command(&args)?;
    cmd.run()
}
