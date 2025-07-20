use super::RunnableCommand;
use clap::Args;

#[derive(Args)]
pub struct RunCommand {
    session_name: String,

    /// Show commands without executing them
    #[arg(short, long)]
    show: bool,

    /// Delay (in milliseconds) between commands
    #[arg(long, short, default_value_t = 0, value_name = "ms")]
    delay: u64,
}

impl RunnableCommand for RunCommand {
    fn run(&self) -> Result<(), &'static str> {
        todo!("Implement the run function for Run command");
        Ok(())
    }
}

impl RunCommand {
    pub fn new(session_name: String, show: bool, delay: u64) -> Self {
        Self {
            session_name,
            show,
            delay,
        }
    }
}
