use crate::errors::ReplayError;
use crate::paths;
use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

#[derive(Default, Serialize, Deserialize)]
pub struct Session {
    pub description: Option<String>,
    pub id: String,
    #[cfg(test)]
    pub timestamp: chrono::DateTime<Utc>,
    #[cfg(not(test))]
    timestamp: chrono::DateTime<Utc>,
    user: String,
    commands: Vec<String>,
}
#[derive(Deserialize, Debug)]
pub struct MetaData {
    pub description: Option<String>,
    pub timestamp: chrono::DateTime<Utc>,
    #[serde(rename = "commands", deserialize_with = "first_two_commands")]
    pub first_commands: Vec<String>,
}

fn first_two_commands<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let all: Vec<String> = Vec::deserialize(deserializer)?;
    Ok(all
        .into_iter()
        .take(2)
        .map(|cmd| cmd.replace("\r", ""))
        .collect())
}
pub struct SessionIndexFile;

impl SessionIndexFile {
    pub fn get_path() -> PathBuf {
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

    /// Read the file by the end and give the byte offset of the nth line
    fn get_nth_line_offset(n: u32) -> Result<u64, ReplayError> {
        let mut file = Self::open_file()?;
        let mut offset = file.seek(SeekFrom::End(0))?;
        let mut buf = [0u8; 1];
        let mut newlines_count = 0;
        let mut target_offset: u64 = 0;

        while offset > 0 {
            offset -= 1;
            file.seek(SeekFrom::Start(offset))?;
            file.read_exact(&mut buf)?;
            if buf[0] == b'\n' {
                newlines_count += 1;
                if newlines_count == n + 2 {
                    target_offset = offset + 1;
                    break;
                }
            }
        }

        if newlines_count == 1 {
            return Ok(0);
        }

        if newlines_count == 0 {
            return Err(ReplayError::SessionError(String::from(
                "No replay entries found",
            )));
        }

        Ok(target_offset)
    }

    /// Read the line starting at a given byte position
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

    /// Get the nth session id and remove it from the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn remove_session_id(n: u32) -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        let line_start_offset = Self::get_nth_line_offset(n)?;

        let session_id = Self::read_line_at(line_start_offset)?;

        // Calculate the position of next line
        // Note: `read_line_at` returns the line without the trailing newline character,
        // so `session_id.len()` does not include the newline. The actual line in the file
        // is `session_id.len() + 1` bytes (session ID plus '\n'), so this calculation is correct.
        let next_line_offset = line_start_offset + session_id.len() as u64 + 1;

        // Read the end of the file after this line
        let mut rest = Vec::new();
        file.seek(SeekFrom::Start(next_line_offset))?;
        file.read_to_end(&mut rest)?;

        // Truncate and rewrite the rest
        file.set_len(line_start_offset)?;
        file.seek(SeekFrom::Start(line_start_offset))?;
        file.write_all(&rest)?;
        file.flush()?;

        Ok(session_id)
    }

    pub fn get_session_id(index: u32) -> Result<String, ReplayError> {
        let line_offset = Self::get_nth_line_offset(index)?;
        Self::read_line_at(line_offset)
    }

    /// Get the last session id without modifying the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn peek_session_id() -> Result<String, ReplayError> {
        let line_offset = Self::get_nth_line_offset(0)?;
        Self::read_line_at(line_offset)
    }

    /// Get the last session id and remove it from the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn pop_session_id() -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        let line_offset = Self::get_nth_line_offset(0)?;
        let session_id = Self::read_line_at(line_offset)?;

        file.set_len(line_offset)?; // truncate at the start of last line
        file.flush()?;

        Ok(session_id)
    }
}

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

    pub fn add_command(&mut self, cmd_raw: Vec<u8>) {
        self.commands
            .push(String::from_utf8_lossy(&cmd_raw).to_string());
    }

    fn load_from_files<T: DeserializeOwned>(session_id: &str) -> Result<T, ReplayError> {
        // Try compressed .zst first
        let zst_path = Session::get_session_path(session_id, "zst");
        if zst_path.exists() {
            let file = File::open(zst_path)?;
            let decoder = zstd::Decoder::new(file)?;
            let data = serde_json::from_reader(decoder)?;
            return Ok(data);
        }

        // Fallback to plain .json
        let json_path = Session::get_session_path(session_id, "json");
        let file = File::open(json_path)?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader)?;
        Ok(data)
    }

    pub fn load_session_by_index(index: u32) -> Result<Self, ReplayError> {
        let session_id = SessionIndexFile::get_session_id(index)?;
        Session::load_from_files(&session_id)
    }

    pub fn load_metadata(session_id: &str) -> Result<MetaData, ReplayError> {
        Session::load_from_files(session_id)
    }

    pub fn load_last_session() -> Result<Self, ReplayError> {
        Self::load_session_by_index(0)
    }

    pub fn save_session(&self, compress: bool) -> Result<(), ReplayError> {
        if compress {
            let file = std::fs::File::create(Self::get_session_path(&self.id, "zst"))?;
            let mut encoder = zstd::Encoder::new(file, DEFAULT_COMPRESSION_LEVEL)?;
            serde_json::to_writer_pretty(&mut encoder, &self)?;
            encoder.finish()?;
        } else {
            let json = serde_json::to_string_pretty(&self)?;
            std::fs::write(Self::get_session_path(&self.id, "json"), json)?;
        }

        SessionIndexFile::push_session(&self.id)?;
        Ok(())
    }

    pub fn iter_commands(&self) -> impl Iterator<Item = &String> + '_ {
        // We use impl Iterator to not have to declare RecordedCommand public
        self.commands.iter()
    }
    pub fn get_session_path(id: &str, extension: &str) -> PathBuf {
        paths::get_sessions_dir().join(format!("{}.{}", id, extension))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serial_test::serial;

    pub fn setup() {
        let _ = std::fs::remove_file(SessionIndexFile::get_path());
    }

    #[test]
    #[serial]
    fn test_session_creation() {
        setup();
        let session = Session::new(Some("test session".into())).unwrap();
        assert_eq!(session.description, Some("test session".into()));
    }

    #[test]
    #[serial]
    fn test_session_saving() {
        setup();
        let session = Session::new(Some("test session".into())).unwrap();

        // Non-compressed saving
        session.save_session(false).unwrap();
        assert!(std::path::Path::new(&Session::get_session_path(&session.id, "json")).exists());

        // Compressed saving
        session.save_session(true).unwrap();
        assert!(std::path::Path::new(&Session::get_session_path(&session.id, "zst")).exists());
    }

    #[test]
    #[serial]
    fn test_session_index_file() {
        setup();
        let session_1 = Session::new(Some("test session 1".into())).unwrap();
        session_1.save_session(true).unwrap();
        assert_eq!(SessionIndexFile::peek_session_id().unwrap(), session_1.id);

        let session_2 = Session::new(Some("test session 2".into())).unwrap();
        session_2.save_session(true).unwrap();
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
