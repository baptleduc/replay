use serde_json::Error as SerdeError;
use thiserror::Error;

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

    #[error("Invalid Session Name: {0}")]
    InvalidSessionName(String),

    #[error("Unknown replay error")]
    Unknown,
}
