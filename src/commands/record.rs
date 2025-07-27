use super::RunnableCommand;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
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

#[cfg(test)]
impl RecordCommand {
    pub fn new(desc: Option<String>) -> Self {
        RecordCommand {
            session_description: desc,
        }
    }
}
