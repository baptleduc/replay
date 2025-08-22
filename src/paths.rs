#[cfg(test)]
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;

pub fn get_replay_dir() -> PathBuf {
    #[cfg(not(test))]
    let dir = dirs::home_dir().unwrap().join(".replay");

    #[cfg(test)]
    let dir = temp_dir().join(".replay");

    fs::create_dir_all(&dir).expect("Failed to create .replay directory");
    dir
}

pub fn get_sessions_dir() -> PathBuf {
    let dir = get_replay_dir().join("sessions");

    fs::create_dir_all(&dir).expect("Failed to create sessions directory");
    dir
}

#[cfg(test)]
pub mod tests {
    use super::*;
    pub fn clear_replay_dir() {
        let dir = get_replay_dir();
        fs::remove_dir_all(&dir).expect("Failed to clear .replay directory");
    }
}
