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
    /// Session name in the form replay@{index}
    #[arg(
        value_name = "session_name",
        value_parser = RunCommand::parse_session_index
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
            Some(index) => Session::load_session_by_index(*index)?,
            None => Session::load_last_session()?,
        };
        let commands: String = session.iter_commands().map(|s| s.as_str()).collect();
        let input = RawModeReader::new(commands.as_bytes());
        let output = stdout();
        run_internal(input, output, false, None)?;
        Ok(())
    }
}

impl RunCommand {
    fn parse_session_index(s: &str) -> Result<u32, String> {
        s.strip_prefix("replay@{")
            .and_then(|rest| rest.strip_suffix('}'))
            .ok_or_else(|| {
                format!(
                    "Session name must be of the form replay@{{index}}, got '{}'",
                    s
                )
            })?
            .parse::<u32>()
            .map_err(|_| format!("Invalid session index in '{}'", s))
    }

    #[cfg(test)]
    /// Get Read-Only Access to the Session Index
    pub fn get_session_index(&self) -> Option<&u32> {
        self.session_index.as_ref()
    }

    #[cfg(test)]
    /// Get Read-Only Access to show
    pub fn get_show(&self) -> bool {
        self.show
    }

    #[cfg(test)]
    /// Get Read-Only Access to delay
    pub fn get_delay(&self) -> u64 {
        self.delay
    }
}
