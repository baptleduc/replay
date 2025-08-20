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
    match CliParser::try_parse_from(args) {
        Ok(parsed_command) => Ok(parsed_command.command),
        Err(err) => Err(ReplayError::ClapError(err)),
    }
}

pub fn validate_session_description(s: &str) -> Result<String, ReplayError> {
    if s.len() > 30 {
        return Err(ReplayError::SessionError(String::from(
            "Session description is too long (max 30 chars)",
        )));
    }

    if s.parse::<i32>().is_ok() {
        return Err(ReplayError::SessionError(String::from(
            "Session description cannot be an integer",
        )));
    }

    Ok(String::from(s))
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
        let parsed_command = parse_command(&args).unwrap();
        if let CliCommand::Record(record_cmd) = parsed_command {
            assert_eq!(
                record_cmd.session_description(),
                &Some(String::from("\"test_valid_record_command\""))
            );
        } else {
            panic!("Expected Record command, got something else");
        }
    }

    #[test]
    fn test_valid_record_command_no_description() {
        let args = [
            String::from("replay"),
            String::from("record"),
        ];
        let parsed_command = parse_command(&args).unwrap();
        if let CliCommand::Record(record_cmd) = parsed_command {
            assert_eq!(record_cmd.session_description(), &None);
        } else {
            panic!("Expected Record command, got something else");
        }
    }

    #[test]
    fn test_valid_run_command() {
        let args = [
            String::from("replay"),
            String::from("run"),
            String::from("\"test_valid_run_command\""),
            String::from("--show"),
            String::from("--delay"),
            String::from("1"),
        ];
        let expected_command = CliCommand::Run(run::RunCommand::new(
            Some(String::from("\"test_valid_run_command\"")),
            true,
            1,
        ));
        assert_eq!(expected_command, parse_command(&args).unwrap())
    }

    #[test]
    fn test_invalid_run_command() {
        let args = [
            String::from("replay"),
            String::from("run"),
            String::from("\"test_valid_run_command\""),
            String::from("--show"),
            String::from("--delay"),
            String::from("is_not_a_number"),
        ];

        let res = parse_command(&args);
        assert!(matches!(res, Err(ReplayError::ClapError(_))));
    }

    #[test]
    fn test_invalid_record_command() {
        let args = [
            String::from("replay"),
            String::from("record"),
            String::from("65"), // An integer cannot be a session name
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
