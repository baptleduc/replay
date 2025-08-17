use std::io::{stdin, stdout};

use super::RunnableCommand;
use crate::args::validate_session_description;
use crate::errors::ReplayError;
use crate::pty::run_internal;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct RecordCommand {
    #[arg(value_parser=validate_session_description)]
    session_description: Option<String>,
}

impl RunnableCommand for RecordCommand {
    fn run(&self) -> Result<(), ReplayError> {
        let reader = stdin();
        let writer = stdout();
        run_internal(reader, writer, true, self.session_description.clone())
    }
}

#[cfg(test)]
impl RecordCommand {
    pub fn new(desc: Option<String>) -> Self {
        RecordCommand {
            session_description: desc,
        }
    }
}
