//! # Commands
//!
//! `Commands` is the module to launch the corresponding subcommands
//! Depending on the flags passed in the CLI parameters

use crate::errors::Result;

// Add commands mod below using pub mod ...
pub mod clear;
pub mod drop;
pub mod list;
pub mod record;
pub mod run;

/// This trait is the common runner trait
pub trait RunnableCommand {
    /// A runner method is needed for each command
    fn run(&self) -> Result<()>;
}
