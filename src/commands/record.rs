use std::io::{stdin, stdout};

use super::RunnableCommand;
use crate::errors::ReplayResult;
use crate::pty::{run_internal, RecordConfig};
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct RecordCommand {
    #[arg(value_parser=RecordCommand::validate_session_description)]
    session_description: Option<String>,

    /// Disable default file compression
    #[arg(long)]
    no_compression: bool,
}
impl RunnableCommand for RecordCommand {
    fn run(&self) -> ReplayResult<()> {
        let reader = stdin();
        let writer = stdout();
        run_internal(
            reader,
            writer,
            RecordConfig {
                record_input: true,
                ..Default::default()
            },
        )
    }
}

impl RecordCommand {
    #[cfg(test)]
    pub fn new(desc: Option<String>, no_compression: bool) -> Self {
        RecordCommand {
            session_description: desc,
            no_compression,
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
