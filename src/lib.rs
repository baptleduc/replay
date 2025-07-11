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

use args::{CliCommand, CliParser};
use clap::Parser;
use commands::{CommandRunner, record::RecordCommand, run::RunCommand};

/// Entrypoint called by the binary.
/// Parses CLI arguments and run the appropriate command.
pub fn run() -> Result<(), &'static str> {
    // We parse the CLI using our CliParser
    let cli_args = CliParser::parse();

    let command = match cli_args.command {
        Some(cmd) => cmd,
        None => {
            return Err("You didn't specify a command, perhaps you should try `--help`");
        }
    };

    // Then we launch the corresponding command
    let cmd: Box<dyn CommandRunner> = match command {
        CliCommand::Run {
            session_name,
            show,
            delay,
            from,
            to,
        } => Box::new(RunCommand::new(session_name, show, delay, from, to)),
        CliCommand::Record { session_name } => Box::new(RecordCommand::new(session_name)),
    };
    cmd.run()?;
    Ok(())
}
