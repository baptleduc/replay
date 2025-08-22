use super::RunnableCommand;
use crate::errors::ReplayError;
use crate::paths;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct ClearCommand {}

impl RunnableCommand for ClearCommand {
    fn run(&self) -> Result<(), ReplayError> {
        paths::clear_replay_dir()?;
        println!("Sessions cleared");
        Ok(())
    }
}
