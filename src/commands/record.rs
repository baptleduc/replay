use super::RunnableCommand;
use clap::Args;

#[derive(Args)]
pub struct RecordCommand {
    /// Optional session name
    session_name: Option<String>,
}

impl RunnableCommand for RecordCommand {
    fn run(&self) -> Result<(), &'static str> {
        todo!("Implement the running function");
        Ok(())
    }
}

impl RecordCommand {
    pub fn new(session_name: Option<String>) -> Self {
        Self { session_name }
    }
}
