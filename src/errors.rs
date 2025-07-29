use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error(transparent)]
    ClapError(#[from] clap::error::Error),
    #[error("Session error: {0}")]
    SessionError(String),
    #[error("Unknown replay error")]
    Unknown,
}
