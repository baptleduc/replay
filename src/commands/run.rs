use super::RunnableCommand;
use super::session::Session;
use clap::Args;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct RunCommand {
    session_name: Option<String>,
    /// Show commands without executing them
    #[arg(short, long)]
    show: bool,

    /// Delay (in milliseconds) between commands
    #[arg(long, short, default_value_t = 0, value_name = "ms")]
    delay: u64,
}

impl RunnableCommand for RunCommand {
    fn run(&self) -> Result<(), &'static str> {
        let session: Session = match &self.session_name {
            Some(name) => Session::load_session(name)?,
            None => Session::load_last_session()?,
        };

        for command in Session::iter_commands(&session) {
            todo!("Use pipe to pass command to shell thread");
        }

        Ok(())
    }
}
