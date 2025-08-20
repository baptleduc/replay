use std::io::{stdin, stdout};

use super::RunnableCommand;
use crate::errors::ReplayError;
use crate::pty::run_internal;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct RecordCommand {
    #[arg(value_parser=RecordCommand::validate_session_description)]
    session_description: Option<String>,
}

impl RunnableCommand for RecordCommand {
    fn run(&self) -> Result<(), ReplayError> {
        let reader = stdin();
        let writer = stdout();
        run_internal(reader, writer, true, self.session_description.clone())
    }
}

impl RecordCommand {
    #[cfg(test)]
    pub fn new(desc: Option<String>) -> Self {
        RecordCommand {
            session_description: desc,
        }
    }

    fn validate_session_description(s: &str) -> Result<String, String> {
        if s.len() < 10 {
            return Err(String::from(
                "Session description is too short (min 10 chars)",
            ));
        }
        if s.len() > 80 {
            return Err(String::from(
                "Session description is too long (max 80 chars)",
            ));
        }

        if s.parse::<i32>().is_ok() {
            return Err(String::from("Session description cannot be an integer"));
        }

        Ok(String::from(s))
    }
}
