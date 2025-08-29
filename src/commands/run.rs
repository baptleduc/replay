//! RunCommand: Replay a recorded session with optional delay and dry-run.

use super::RunnableCommand;
use crate::args;
use crate::errors::ReplayError;
use crate::pty::{RawModeReader, run_internal};
use crate::session::Session;
use clap::{Args, value_parser};
use std::io::stdout;

/// CLI command to run a recorded session.
#[derive(Args, PartialEq, Eq, Debug)]
pub struct RunCommand {
    /// Session name in the form replay@{index}
    #[arg(
        value_name = "session_name",
        default_value = "replay@{0}",
        value_parser = args::parse_session_index
    )]
    session_index: u32,

    /// Show commands without executing them
    #[arg(short, long)]
    show: bool,

    /// Delay in milliseconds between each character during replay typing.
    /// Must be at least 10 ms.
    #[arg(long, short, default_value_t = 10, value_name = "ms", value_parser = value_parser!(u64).range(10..))]
    delay: u64,
}

impl RunnableCommand for RunCommand {
    fn run(&self) -> Result<(), ReplayError> {
        let session: Session = Session::load_session_by_index(self.session_index)?;
        if self.show {
            self.show_commands(session)?;
        } else {
            let commands: String = session.iter_commands().collect();
            let input = RawModeReader::with_input_and_delay(
                commands.as_bytes(),
                std::time::Duration::from_millis(self.delay),
            );
            let output = stdout();
            run_internal(input, output, false, None, false)?;
        }
        Ok(())
    }
}

impl RunCommand {
    #[cfg(test)]
    pub fn new(session_index: u32, show: bool, delay: u64) -> Self {
        Self {
            session_index,
            show,
            delay,
        }
    }

    fn show_commands(&self, session: Session) -> Result<(), ReplayError> {
        println!("Commands for session 'replay@{{{}}}':", self.session_index);
        // The last command is always "exit", so we skip printing it
        for (i, cmd) in session
            .iter_commands()
            .take(session.iter_commands().count() - 1)
            .enumerate()
        {
            println!("  {}. {}", i + 1, cmd);
        }
        Ok(())
    }
}
