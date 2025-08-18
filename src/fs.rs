#[cfg(test)]
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;

pub fn get_replay_dir() -> PathBuf {
    #[cfg(not(test))]
    let dir = dirs::home_dir().unwrap().join(".replay");

    #[cfg(test)]
    let dir = temp_dir().join(".replay");

    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create .replay directory");
    }
    dir
}

pub fn get_sessions_dir() -> PathBuf {
    let dir = get_replay_dir().join("sessions");

    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create sessions directory");
    }
    dir
}
