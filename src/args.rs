//! # Args
//!
//! `Args` is the parsing module used by our main library
//! It will ensure we get the correct args and then return
//! a correct Structure to run the corresponding commands
use crate::commands::{RunnableCommand, record, run};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliParser {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Run a replay on the specified session
    Run(run::RunCommand),

    /// Record a new session of shell commands
    /// if a session name is provided, it will be used to labael the recording
    Record(record::RecordCommand),
}

impl CliCommand {
    pub fn run(&self) -> Result<(), &'static str> {
        match self {
            CliCommand::Run(cmd) => cmd.run(),
            CliCommand::Record(cmd) => cmd.run(),
        }
    }
}
