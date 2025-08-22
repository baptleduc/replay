use crate::errors::ReplayError;
#[cfg(test)]
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;

fn replay_dir_path() -> PathBuf {
    #[cfg(test)]
    {
        temp_dir().join(".replay")
    }
    #[cfg(not(test))]
    {
        dirs::home_dir().unwrap().join(".replay")
    }
}

pub fn replay_dir() -> PathBuf {
    let dir = replay_dir_path();
    fs::create_dir_all(&dir).expect("Failed to create .replay directory");
    dir
}

pub fn session_dir() -> PathBuf {
    let dir = replay_dir().join("sessions");

    fs::create_dir_all(&dir).expect("Failed to create sessions directory");
    dir
}

pub fn clear_replay_dir() -> Result<(), ReplayError> {
    let dir_path = replay_dir_path();
    if dir_path.exists() {
        fs::remove_dir_all(&dir_path)?;
    }
    Ok(())
}
