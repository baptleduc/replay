//! # Args
//!
//! `Args` is the parsing module used by our main library
//! It will ensure we get the correct args and then return
//! a correct Structure to run the corresponding commands
use crate::{
    commands::{RunnableCommand, clear, drop, list, record, run},
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

    /// List all the sessions recorded
    List(list::ListCommand),

    /// Drop a specified session, last session if not specified
    Drop(drop::DropCommand),
    /// Drop all the sessions recorded
    Clear(clear::ClearCommand),
}

impl CliCommand {
    pub fn run(&self) -> Result<(), ReplayError> {
        match self {
            CliCommand::Run(cmd) => cmd.run(),
            CliCommand::Record(cmd) => cmd.run(),
            CliCommand::List(cmd) => cmd.run(),
            CliCommand::Drop(cmd) => cmd.run(),
            CliCommand::Clear(cmd) => cmd.run(),
        }
    }
}

pub fn parse_command(args: &[String]) -> Result<CliCommand, ReplayError> {
    let cli_command = CliParser::try_parse_from(args)?;
    Ok(cli_command.command)
}

pub fn parse_session_index(s: &str) -> Result<u32, String> {
    s.strip_prefix("replay@{")
        .and_then(|rest| rest.strip_suffix('}'))
        .ok_or_else(|| {
            format!(
                "Session name must be of the form replay@{{index}}, got '{}'",
                s
            )
        })?
        .parse::<u32>()
        .map_err(|_| format!("Invalid session index in '{}'", s))
}

// TODO move it as integration tests
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
            String::from("10"),
        ];
        let expected_command = CliCommand::Run(run::RunCommand::new(0, true, 10));
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
            String::from("10"),
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

        // Delay not in the right range
        let args = [
            String::from("replay"),
            String::from("run"),
            String::from("replay@{0}"),
            String::from("--show"),
            String::from("--delay"),
            String::from("1"),
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
