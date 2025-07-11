use super::CommandRunner;
pub struct RunCommand {
    session_name: String,
    show: bool,
    delay: u64,
    from: Option<u64>,
    to: Option<u64>,
}

impl CommandRunner for RunCommand {
    fn run(&self) -> Result<(), &'static str> {
        todo!("Implement the run function for Run command");
        Ok(())
    }
}

impl RunCommand {
    pub fn new(
        session_name: String,
        show: bool,
        delay: u64,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Self {
        Self {
            session_name,
            show,
            delay,
            from,
            to,
        }
    }
}
