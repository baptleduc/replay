//! # Args
//!
//! `Args` is the parsing module used by our main library
//! It will ensure we get the correct args and then return
//! a correct Structure to run the corresponding commands
use crate::{
    commands::{RunnableCommand, record, run},
    errors::ReplayError,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliParser {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum CliCommand {
    /// Run a specified session, last session if not specified
    Run(run::RunCommand),

    /// Record a new session of shell commands
    Record(record::RecordCommand),
}

impl CliCommand {
    pub fn run(&self) -> Result<(), ReplayError> {
        match self {
            CliCommand::Run(cmd) => cmd.run(),
            CliCommand::Record(cmd) => cmd.run(),
        }
    }
}

pub fn parse_command(args: &[String]) -> Result<CliCommand, ReplayError> {
    let cli_command = CliParser::try_parse_from(args)?;
    Ok(cli_command.command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_command() {
        let args = [
            String::from("replay"),
            String::from("record"),
            String::from("\"test_valid_record_command\""),
        ];

        let expected_command = CliCommand::Record(record::RecordCommand::new(
            Some(String::from("\"test_valid_record_command\"")),
            false,
        ));
        assert_eq!(expected_command, parse_command(&args).unwrap())
    }

    #[test]
    fn test_valid_run_command() {
        let args = [
            String::from("replay"),
            String::from("run"),
            String::from("replay@{0}"),
            String::from("--show"),
            String::from("--delay"),
            String::from("1"),
        ];
        let expected_command = CliCommand::Run(run::RunCommand::new(Some(0), true, 1));
        assert_eq!(expected_command, parse_command(&args).unwrap());

        // `replay run` is a valid command, it runs the last session by default
        let args = [String::from("replay"), String::from("run")];
        let expected_command = CliCommand::Run(run::RunCommand::new(None, false, 0));
        assert_eq!(expected_command, parse_command(&args).unwrap());
    }

    #[test]
    fn test_invalid_run_command() {
        // Invalid session name
        let args = [
            String::from("replay"),
            String::from("run"),
            String::from("invalid_session_name"),
            String::from("--show"),
            String::from("--delay"),
            String::from("1"),
        ];
        let res = parse_command(&args);
        assert!(matches!(res, Err(ReplayError::ClapError(_))));

        // Delay as a char
        let args = [
            String::from("replay"),
            String::from("run"),
            String::from("replay@{0}"),
            String::from("--show"),
            String::from("--delay"),
            String::from("a"),
        ];
        let res = parse_command(&args);
        assert!(matches!(res, Err(ReplayError::ClapError(_))));
    }

    #[test]
    fn test_invalid_record_command() {
        // Too short session description
        let args = [
            String::from("replay"),
            String::from("record"),
            String::from("to short"),
        ];
        let res = parse_command(&args);
        assert!(matches!(res, Err(ReplayError::ClapError(_))));

        // Too long session description
        let args = [
            String::from("replay"),
            String::from("record"),
            String::from(
                "this session description is way too long and exceeds the maximum length of eighty characters",
            ),
        ];
        let res = parse_command(&args);
        assert!(matches!(res, Err(ReplayError::ClapError(_))));

        // Invalid session description
        let args = [
            String::from("replay"),
            String::from("record"),
            String::from("1234567890"),
        ];
        let res = parse_command(&args);
        assert!(matches!(res, Err(ReplayError::ClapError(_))));
    }

    #[test]
    fn test_invalid_command() {
        let args = [
            String::from("replay"),
            String::from("invalid"), // Not a valid command
        ];
        let res = parse_command(&args);
        assert!(matches!(res, Err(ReplayError::ClapError(_))));
    }
}
