use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error(transparent)]
    ClapError(#[from] clap::error::Error),
    #[error("Session error: {0}")]
    SessionError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error during PTY handling: {0}")]
    Pty(#[from] anyhow::Error),

    #[error("Unknown replay error")]
    Unknown,
}
