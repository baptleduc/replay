use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("Clap message error")]
    ClapError(#[from] clap::error::Error),
    #[error("Session error")]
    SessionError(String),
    #[error("Unknown replay error")]
    Unknown,
}
