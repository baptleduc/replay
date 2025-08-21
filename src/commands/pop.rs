use super::RunnableCommand;
use crate::errors::ReplayError;
use crate::session::Session;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct PopCommand {}

impl RunnableCommand for PopCommand {
    fn run(&self) -> Result<(), ReplayError> {
        Session::remove_last_session()?;
        Ok(())
    }
}
