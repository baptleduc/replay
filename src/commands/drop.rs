use super::RunnableCommand;
use crate::errors::ReplayError;
use crate::parsers;
use crate::session::Session;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct DropCommand {
    #[arg(
        value_name = "session_name",
        value_parser = parsers::parse_session_index
    )]
    session_index: u32,
}

impl RunnableCommand for DropCommand {
    fn run(&self) -> Result<(), ReplayError> {
        Session::remove_session(self.session_index)?;
        Ok(())
    }
}
