//! RunCommand: Replay a recorded session with optional delay and dry-run.

use super::RunnableCommand;
use crate::errors::ReplayError;
use crate::session::Session;
use clap::Args;

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
        match &self.session_index {
            Some(index) => Session::run_session_by_index(*index)?,
            None => Session::run_last_session()?,
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
}
