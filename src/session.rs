use crate::errors::ReplayError;
use crate::paths;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

#[cfg(not(test))]
use sha2::{Digest, Sha256};

#[derive(Default, Serialize, Deserialize)]
pub struct Session {
    pub description: Option<String>,
    pub id: String,
    timestamp: chrono::DateTime<Utc>,
    user: String,
    pub commands: Vec<String>,
}
struct SessionIndexFile;

impl SessionIndexFile {
    fn get_path() -> PathBuf {
        paths::get_replay_dir().join("session_idx")
    }

    fn open_file() -> Result<std::fs::File, ReplayError> {
        Ok(std::fs::OpenOptions::new()
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

    fn get_nth_line_pos(n: u32) -> Result<u64, ReplayError> {
        let mut file = Self::open_file()?;
        let mut pos = file.seek(SeekFrom::End(0))?;
        let mut buf = [0u8; 1];
        let mut newlines_found = 0;
        let mut line_offset: u64 = 0;

        while pos > 0 {
            pos -= 1;
            file.seek(SeekFrom::Start(pos))?;
            file.read_exact(&mut buf)?;
            if buf[0] == b'\n' {
                newlines_found += 1;
                if newlines_found == n + 2 {
                    line_offset = pos + 1;
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

        Ok(line_offset)
    }

    /// Read the line starting at a given byte offset
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    fn read_line_at(offset: u64) -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        file.seek(SeekFrom::Start(offset))?;
        // We use a BufReader for the `read_until()` func
        let mut reader = BufReader::new(file);
        let mut buf = Vec::new();
        // We read until a \n instead of reading the entire file
        reader.read_until(b'\n', &mut buf)?;
        let line = String::from_utf8_lossy(&buf)
            .trim_end_matches('\n')
            .to_string();
        Ok(line)
    }

    /// Get the nth session id without modifying the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn get_nth_session_id(n: u32) -> Result<String, ReplayError> {
        let offset = Self::get_nth_line_pos(n)?;
        Self::read_line_at(offset)
    }

    /// Get the nth session id and remove it from the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn drop_session_id(n: u32) -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        let start_offset = Self::get_nth_line_pos(n)?;

        let session_id = Self::read_line_at(start_offset)?;

        // Calculate the offset off the rest of the file
        let next_offset = start_offset + session_id.len() as u64 + 1;

        // Read the end of the file after this line
        let mut rest = Vec::new();
        file.seek(SeekFrom::Start(next_offset))?;
        file.read_to_end(&mut rest)?;

        // Truncate and rewrite the rest
        file.set_len(start_offset)?;
        file.seek(SeekFrom::Start(start_offset))?;
        file.write_all(&rest)?;
        file.flush()?;

        Ok(session_id)
    }

    /// Get the last session id without modifying the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn peek_session_id() -> Result<String, ReplayError> {
        let offset = Self::get_nth_line_pos(0)?;
        Self::read_line_at(offset)
    }

    /// Get the last session id and remove it from the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn pop_session_id() -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        let offset = Self::get_nth_line_pos(0)?;
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
        let replay_index = Self::get_replay_index(session_name)?;
        let id = SessionIndexFile::get_nth_session_id(replay_index)?;
        let session_file = File::open(Self::get_session_path(&id))?;
        let loaded_session: Session = serde_json::from_reader(session_file)?;
        Ok(loaded_session)
    }

    fn get_replay_index(session_name: &str) -> Result<u32, ReplayError> {
        session_name
            .strip_prefix("replay@{")
            .and_then(|s| s.strip_suffix('}'))
            .ok_or_else(|| ReplayError::InvalidSessionName(session_name.to_string()))?
            .parse()
            .map_err(|_| ReplayError::InvalidSessionName(session_name.to_string()))
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
    pub fn get_session_path(id: &str) -> PathBuf {
        paths::get_sessions_dir().join(format!("{}.json", id))
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
