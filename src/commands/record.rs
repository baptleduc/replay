use super::CommandRunner;

pub struct RecordCommand {
    session_name: Option<String>,
}

impl CommandRunner for RecordCommand {
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
