//! RunCommand: Replay a recorded session with optional delay and dry-run.

use super::RunnableCommand;
use crate::errors::ReplayError;
use crate::pty::{RawModeReader, run_internal};
use crate::session::Session;
use clap::Args;
use std::io::stdout;

/// CLI command to run a recorded session.
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
    fn run(&self) -> Result<(), ReplayError> {
        let session: Session = match &self.session_name {
            Some(name) => Session::load_session(name)?,
            None => Session::load_last_session()?,
        };
        let commands: String = session.iter_commands().map(|s| s.as_str()).collect();
        let input = RawModeReader::new(commands.as_bytes());
        let output = stdout();
        run_internal(input, output, false, None)?;
        Ok(())
    }
}

#[cfg(test)]
impl RunCommand {
    pub fn new(session_name: Option<String>, show: bool, delay: u64) -> Self {
        RunCommand {
            session_name,
            show,
            delay,
        }
    }
}
