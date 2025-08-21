use super::RunnableCommand;
use crate::args;
use crate::errors::ReplayError;
use crate::session::Session;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct DropCommand {
    #[arg(
        value_name = "session_name",
        value_parser = args::parse_session_index
    )]
    session_index: Option<u32>,
}

impl RunnableCommand for DropCommand {
    fn run(&self) -> Result<(), ReplayError> {
        if let Some(index) = self.session_index {
            Session::remove_session(index)?
        } else {
            Session::remove_last_session()?
        }
        Ok(())
    }
}
