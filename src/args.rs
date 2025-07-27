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
    /// Run a replay on the specified session
    Run(run::RunCommand),

    /// Record a new session of shell commands
    /// if a session name is provided, it will be used to labael the recording
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
        parse_command(&args).unwrap_err();
    }

    #[test]
    fn test_invalid_record_command() {
        let args = [
            String::from("replay"),
            String::from("record"),
            String::from("65"), // An integer cannot be a session name
        ];
        parse_command(&args).unwrap_err();
    }
}
