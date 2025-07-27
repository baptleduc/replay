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

#[derive(Subcommand, PartialEq, Eq, Debug)]
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

pub fn parse_command(args: &[String]) -> Option<CliCommand> {
    CliParser::parse_from(args).command
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;

    #[test]
    fn test_parse_command() {
        // Test with valid record command : `replay record "test1"`
        let args = [
            String::from("replay"),
            String::from("record"),
            String::from("\"test1\""),
        ];
        let expected_command =
            CliCommand::Record(record::RecordCommand::new(Some(String::from("\"test1\""))));
        match parse_command(&args) {
            Some(cmd) => assert_eq!(cmd, expected_command),
            None => panic!("The parsing function from clap is not working"),
        };
    }
}
