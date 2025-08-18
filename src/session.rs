use crate::errors::ReplayError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::env;

#[cfg(not(test))]
use std::{path::Path};

#[cfg(test)]
use std::env::temp_dir;

#[derive(Default, Serialize, Deserialize)]
pub struct Session {
    pub description: Option<String>,
    pub id: String,
    timestamp: chrono::DateTime<Utc>,
    user: String,
    commands: Vec<String>,
}
struct SessionIndexFile;

impl SessionIndexFile {
    fn get_path() -> String {
        format!(
            "{}/.replay/session_idx",
            env::var("HOME").unwrap_or_else(|_| String::from("/home/user"))
        )
    }

    fn open_file() -> Result<std::fs::File, ReplayError> {
        Ok(std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .append(true)
            .open(Self::get_path())?)
    }

    pub fn push_session(session_id: &str) -> Result<(), ReplayError> {
        let mut file = Self::open_file()?;
        writeln!(file, "{}", session_id)?;
        Ok(())
    }

    fn get_last_line_offset() -> Result<u64, ReplayError> {
        let mut file = Self::open_file()?;
        let mut pos = file.seek(SeekFrom::End(0))?;

        let mut buf = [0u8; 1];
        let mut newlines_found = 0;
        let mut last_line_offset = 0;

        // Read the file in reverse to find the last two newlines
        while pos > 0 {
            pos -= 1;
            file.seek(SeekFrom::Start(pos))?;
            file.read_exact(&mut buf)?;
            if buf[0] == b'\n' {
                newlines_found += 1;
                if newlines_found == 2 {
                    last_line_offset = pos + 1; // start of the last line
                    break;
                }
            }
        }

        if newlines_found == 1 {
            return Ok(0);
        }

        if newlines_found == 0 {
            return Err(ReplayError::SessionError(String::from(
                "No replay entries found",
            )));
        }

        Ok(last_line_offset)
    }

    /// Read the line starting at a given byte offset
    fn read_line_at(offset: u64) -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        file.seek(SeekFrom::Start(offset))?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.trim_end_matches('\n').to_string())
    }

    /// Get the last session id without modifying the file
    pub fn peek_session_id() -> Result<String, ReplayError> {
        let offset = Self::get_last_line_offset()?;
        Self::read_line_at(offset)
    }

    /// Get the last session id and remove it from the file
    pub fn pop_session_id() -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        let offset = Self::get_last_line_offset()?;
        let session_id = Self::read_line_at(offset)?;

        file.set_len(offset)?; // truncate at the start of last line
        file.flush()?;

        Ok(session_id)
    }
}

#[cfg(test)]
pub const TEST_ID: &str = "test_session";

impl Session {
    pub fn new(description: Option<String>) -> Result<Self, ReplayError> {
        Self::ensure_sessions_dir_exists()?;
        let user = whoami::username();
        let timestamp = Utc::now();
        Ok(Self {
            commands: Vec::new(),
            id: Self::generate_id(&description, &timestamp, &user),
            description,
            timestamp,
            user,
        })
    }

    #[cfg(not(test))]
    fn generate_id(
        description: &Option<String>,
        timestamp: &chrono::DateTime<Utc>,
        user: &str,
    ) -> String {
        let mut hasher = Sha256::new();

        hasher.update(user.as_bytes());
        if let Some(desc) = description {
            hasher.update(desc.as_bytes());
        }
        hasher.update(timestamp.to_rfc3339().as_bytes());

        format!("{:x}", hasher.finalize())
    }

    #[cfg(test)]
    fn generate_id(
        _description: &Option<String>,
        _timestamp: &chrono::DateTime<Utc>,
        _user: &str,
    ) -> String {
        TEST_ID.to_string()
    }

    pub fn add_command(&mut self, cmd_raw: Vec<u8>) {
        self.commands
            .push(String::from_utf8_lossy(&cmd_raw).to_string());
    }

    pub fn to_json(&self) {
        todo!("implement json structure");
    }

    pub fn load_session(session_name: &str) -> Result<Self, ReplayError> {
        let _ = session_name; // TODO: remove
        todo!("Use DEFAULT_SESSION_PATH to load session by its name, and return SessionError")
    }

    pub fn load_last_session() -> Result<Self, ReplayError> {
        todo!("load last session");
    }

    pub fn save_session(&self) -> Result<(), ReplayError> {
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(Self::get_session_path(&self.id), json)?;
        SessionIndexFile::push_session(&self.id)?;
        Ok(())
    }

    pub fn iter_commands(&self) -> impl Iterator<Item = &String> + '_ {
        // We use impl Iterator to not have to declare RecordedCommand public
        self.commands.iter()
    }

    #[cfg(not(test))]
    fn get_sessions_dir() -> PathBuf {
        env::var("HOME")
            .map(|home| Path::new(&home).join(".replay/sessions"))
            .unwrap_or_else(|_| Path::new("/home/user/.replay/sessions").to_path_buf())
    }

    #[cfg(test)]
    fn get_sessions_dir() -> PathBuf {
        temp_dir()
    }

    fn ensure_sessions_dir_exists() -> Result<(), ReplayError> {
        // create the sessions dir if it doesn't already exists
        std::fs::create_dir_all(Self::get_sessions_dir())?;
        Ok(())
    }

    pub fn get_session_path(id: &str) -> PathBuf {
        Self::get_sessions_dir().join(format!("{}.json", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn setup() {
        let _ = std::fs::remove_file(SessionIndexFile::get_path());
    }

    #[test]
    #[serial]
    fn test_session_creation() {
        setup();
        let session = Session::new(Some("test session".into())).unwrap();
        assert_eq!(session.description, Some("test session".into()));
        assert_eq!(session.id, TEST_ID);
    }

    #[test]
    #[serial]
    fn test_session_saving() {
        setup();
        let session = Session::new(Some("test session".into())).unwrap();
        session.save_session().unwrap();
        assert!(std::path::Path::new(&Session::get_session_path(&session.id)).exists());
    }

    #[test]
    #[serial]
    fn test_session_index_file() {
        setup();
        let session_1 = Session::new(Some("test session 1".into())).unwrap();
        session_1.save_session().unwrap();
        assert_eq!(SessionIndexFile::peek_session_id().unwrap(), session_1.id);

        let session_2 = Session::new(Some("test session 2".into())).unwrap();
        session_2.save_session().unwrap();
        assert_eq!(SessionIndexFile::peek_session_id().unwrap(), session_2.id);
        assert_eq!(SessionIndexFile::pop_session_id().unwrap(), session_2.id);
        assert_eq!(SessionIndexFile::pop_session_id().unwrap(), session_1.id);

        // Test popping and peeking from empty index file returns error
        assert!(matches!(
            SessionIndexFile::pop_session_id(),
            Err(ReplayError::SessionError(_))
        ));
        assert!(matches!(
            SessionIndexFile::peek_session_id(),
            Err(ReplayError::SessionError(_))
        ));
    }
}
