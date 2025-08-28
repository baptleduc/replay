use serde_json::Error as SerdeError;
use std::result;
use thiserror::Error;

pub type Result<T> = result::Result<T, ReplayError>;

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("Clap error: {0}")]
    ClapError(#[from] clap::error::Error),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Export error: {0}")]
    ExportError(#[from] SerdeError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error during PTY handling: {0}")]
    Pty(#[from] anyhow::Error),

    #[error("Thread panicked: {0}")]
    ThreadPanic(String),

    #[error("Error while reading line in reverse order: {0}")]
    RevLinesError(#[from] rev_lines::RevLinesError),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Unknown replay error")]
    Unknown,
}
