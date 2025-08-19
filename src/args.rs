//! # Args
//!
//! `Args` is the parsing module used by our main library
//! It will ensure we get the correct args and then return
//! a correct Structure to run the corresponding commands
use crate::{
    commands::{RunnableCommand, list, record, run},
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
}

impl CliCommand {
    pub fn run(&self) -> Result<(), ReplayError> {
        match self {
            CliCommand::Run(cmd) => cmd.run(),
            CliCommand::Record(cmd) => cmd.run(),
            CliCommand::List(cmd) => cmd.run(),
        }
    }
}

pub fn parse_command(args: &[String]) -> Result<CliCommand, ReplayError> {
    match CliParser::try_parse_from(args) {
        Ok(parsed_command) => Ok(parsed_command.command),
        Err(err) => Err(ReplayError::ClapError(err)),
    }
}

// TODO: move in RecordCommand
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
        // TODO: remove new command from RecordCommand and add getters to get field as for RunCommand
        let expected_command = CliCommand::Record(record::RecordCommand::new(Some(String::from(
            "\"test_valid_record_command\"",
        ))));
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

        let parsed_cmd = parse_command(&args).unwrap();
        if let CliCommand::Run(run_cmd) = parsed_cmd {
            assert_eq!(run_cmd.get_session_index(), Some(&0));
            assert!(run_cmd.get_show());
            assert_eq!(run_cmd.get_delay(), 1);
        } else {
            panic!("Expected RunCommand");
        }
    }

    #[test]
    fn test_invalid_run_command() {
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
