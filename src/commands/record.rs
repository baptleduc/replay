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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        // With session name
        let record_cmd = RecordCommand::new(Some("test_session".to_string()));
        assert_eq!(record_cmd.session_name, Some("test_session".to_string()));

        // Without session name
        let record_cmd_none = RecordCommand::new(None);
        assert_eq!(record_cmd_none.session_name, None);
    }
}