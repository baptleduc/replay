use super::RunnableCommand;
use crate::errors::Result;
use crate::paths;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct ClearCommand {}

impl RunnableCommand for ClearCommand {
    fn run(&self) -> Result<()> {
        paths::clear_replay_dir()?;
        println!("Sessions cleared");
        Ok(())
    }
}
