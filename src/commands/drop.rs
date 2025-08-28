use super::RunnableCommand;
use crate::args;
use crate::errors::Result;
use crate::session::Session;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct DropCommand {
    #[arg(
        value_name = "session_name",
        default_value = "replay@{0}",
        value_parser = args::parse_session_index
    )]
    session_index: u32,
}

impl RunnableCommand for DropCommand {
    fn run(&self) -> Result<()> {
        Session::remove_session_by_index(self.session_index)?;
        Ok(())
    }
}
