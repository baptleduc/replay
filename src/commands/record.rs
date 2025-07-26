use super::RunnableCommand;
use clap::Args;

#[derive(Args)]
pub struct RecordCommand {
    #[arg()]
    session_description: Option<String>,
}

impl RunnableCommand for RecordCommand {
    fn run(&self) -> Result<(), &'static str> {
        todo!("Implement the running function");
        Ok(())
    }
}
