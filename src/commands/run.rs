//! RunCommand: Replay a recorded session with optional delay and dry-run.

use super::RunnableCommand;
use crate::args;
use crate::errors::ReplayError;
use crate::pty::{RawModeReader, run_internal};
use crate::session::Session;
use clap::Args;
use std::io::stdout;

/// CLI command to run a recorded session.
#[derive(Args, PartialEq, Eq, Debug)]
pub struct RunCommand {
    /// Session name in the form replay@{index}
    #[arg(
        value_name = "session_name",
        value_parser = args::parse_session_index
    )]
    session_index: Option<u32>,

    /// Show commands without executing them
    #[arg(short, long)]
    show: bool,

    /// Delay (in milliseconds) between commands
    #[arg(long, short, default_value_t = 0, value_name = "ms")]
    delay: u64,
}

impl RunnableCommand for RunCommand {
    fn run(&self) -> Result<(), ReplayError> {
        let session: Session = match &self.session_index {
            Some(idx) => Session::load_session_by_index(*idx)?,
            None => Session::load_last_session()?,
        };
        if self.show {
            self.show_commands(session)?;
        } else {
            let commands: String = session.iter_commands().collect();
            let input = RawModeReader::new(commands.as_bytes());
            let output = stdout();
            run_internal(input, output, false, None, false)?;
        }
        Ok(())
    }
}

impl RunCommand {
    #[cfg(test)]
    pub fn new(session_index: Option<u32>, show: bool, delay: u64) -> Self {
        Self {
            session_index,
            show,
            delay,
        }
    }
}
