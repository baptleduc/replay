//! # Args
//!
//! `Args` is the parsing module used by our main library
//! It will ensure we get the correct args and then return
//! a correct Structure to run the corresponding commands

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliParser {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Run a replay on the specified session
    Run {
        session_name: String,

        /// Show commands without executing them
        #[arg(short, long)]
        show: bool,

        /// Delay (in milliseconds) between commands
        #[arg(long, short, default_value_t = 0, value_name = "ms")]
        delay: u64,

        /// Start replay from command number <N>
        #[arg(long, short, value_name = "N")]
        from: Option<u64>,

        /// Stop replay at command number N
        #[arg(long, short, value_name = "N")]
        to: Option<u64>,
    },

    /// Record a new session of shell commands
    /// if a session name is provided, it will be used to label the recording
    Record {
        /// Optional session name
        session_name: Option<String>,
    },
}
