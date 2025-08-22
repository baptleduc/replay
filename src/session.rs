use crate::errors::ReplayError;
use crate::paths;
use chrono::Utc;
use rev_lines::RevLines;
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
    pub timestamp: chrono::DateTime<Utc>,
    pub user: String,
    pub commands: Vec<String>,
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
struct SessionIndexFile;

impl SessionIndexFile {
    fn get_path() -> PathBuf {
        paths::replay_dir().join("session_idx")
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
    fn get_line_offset_by_index(n: u32) -> Result<u64, ReplayError> {
        let mut file = Self::open_file()?;
        let mut offset = file.seek(SeekFrom::End(0))?;
        let mut buf = [0u8; 1];
        let mut newlines_count = 0;

        if offset == 0 {
            return Err(ReplayError::SessionError("No replay entries found".into()));
        }

        // If the file is not empty, check the last byte
        file.seek(SeekFrom::Start(offset - 1))?;
        file.read_exact(&mut buf)?;

        // If the last byte is a newline, skip it
        // This ensures we don't count an extra empty line at the end
        if buf[0] == b'\n' {
            offset -= 1;
        }

        // Now we scan the file backwards to count newlines
        while offset > 0 {
            // Move back by one byte
            offset -= 1;
            file.seek(SeekFrom::Start(offset))?;
            file.read_exact(&mut buf)?;

            // If we encounter a newline, we found the end of the previous line
            if buf[0] == b'\n' {
                newlines_count += 1;

                // If we've counted enough newlines to reach the target line
                // `n + 1` because index 0 refers to the last line, index 1 to the second last, etc.
                if newlines_count == n + 1 {
                    // The start of the target line is just after this newline
                    return Ok(offset + 1);
                }
            }
        }

        if newlines_count <= n {
            if n == newlines_count {
                return Ok(0);
            } else {
                return Err(ReplayError::SessionError(
                    "Replay index out of range".into(),
                ));
            }
        }

        Err(ReplayError::SessionError("No replay entries found".into()))
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
        let line_start_offset = Self::get_line_offset_by_index(n)?;

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
        let line_offset = Self::get_line_offset_by_index(index)?;
        Self::read_line_at(line_offset)
    }

    /// Get the last session id without modifying the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn peek_session_id() -> Result<String, ReplayError> {
        let line_offset = Self::get_line_offset_by_index(0)?;
        Self::read_line_at(line_offset)
    }

    /// Get the last session id and remove it from the file
    #[allow(dead_code)] // TODO: Remove this when the function will be used
    pub fn pop_session_id() -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        let line_offset = Self::get_line_offset_by_index(0)?;
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

    pub fn remove_last_command(&mut self) -> Option<String> {
        self.commands.pop()
    }

    pub fn get_last_command(&self) -> Option<&String> {
        self.commands.last()
    }

    fn load_from_files<T: DeserializeOwned>(session_id: &str) -> Result<T, ReplayError> {
        // Try compressed .zst first
        let zst_path = Session::get_session_path(session_id, "zst");
        if zst_path.try_exists()? {
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

    pub fn remove_session(index: u32) -> Result<(), ReplayError> {
        let session_id = SessionIndexFile::remove_session_id(index)?;
        let zst_path = Session::get_session_path(&session_id, "zst");
        if zst_path.try_exists()? {
            std::fs::remove_file(zst_path)?;
        } else {
            let json_session_path = Session::get_session_path(&session_id, "json");
            std::fs::remove_file(json_session_path)?;
        }
        Ok(())
    }

    pub fn remove_last_session() -> Result<(), ReplayError> {
        Self::remove_session(0)
    }

    pub fn iter_commands(&self) -> impl Iterator<Item = &str> {
        // We use impl Iterator to not have to declare RecordedCommand public
        self.commands.iter().map(|s| s.as_str())
    }
    pub fn get_session_path(id: &str, extension: &str) -> PathBuf {
        paths::session_dir().join(format!("{}.{}", id, extension))
    }

    pub fn iter_session_ids_rev()
    -> Result<impl Iterator<Item = Result<String, ReplayError>>, ReplayError> {
        let file = SessionIndexFile::open_file()?;
        let rev_lines = RevLines::new(file);
        Ok(rev_lines.map(|line_res| line_res.map_err(ReplayError::from)))
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

    #[test]
    #[serial]
    fn test_session_remove() {
        setup();
        let session1 = Session::new(Some("test session1".into())).unwrap();
        let session2 = Session::new(Some("test session2".into())).unwrap();
        session1.save_session(true).unwrap();
        session2.save_session(true).unwrap();
        Session::remove_session(1).unwrap();
        assert!(
            !std::path::Path::new(&Session::get_session_path(&session1.id, "zst"))
                .try_exists()
                .unwrap()
        );
        let res = SessionIndexFile::get_session_id(1);
        if let Err(ReplayError::SessionError(msg)) = res {
            assert_eq!(msg, "Replay index out of range");
        } else {
            panic!("Expected SessionError, got {:?}", res);
        }
        Session::remove_last_session().unwrap();
        assert!(
            !std::path::Path::new(&Session::get_session_path(&session2.id, "zst"))
                .try_exists()
                .unwrap()
        );
        assert!(matches!(
            SessionIndexFile::get_session_id(2),
            Err(ReplayError::SessionError(ref msg)) if msg == "No replay entries found"
        ));
    }
}
