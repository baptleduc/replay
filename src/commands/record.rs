use super::RunnableCommand;
use crate::args::validate_session_description;
use crate::errors::ReplayError;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct RecordCommand {
    #[arg(value_parser=validate_session_description)]
    session_description: Option<String>,
}

impl RunnableCommand for RecordCommand {
    fn run(&self) -> Result<(), ReplayError> {
        todo!("Implement the running function");
        Ok(())
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
