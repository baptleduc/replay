use crate::errors::ReplayResult;
use crate::paths;
use chrono::Utc;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

mod display;
pub mod index;

pub use display::DisplayMeta;
pub use index::SessionIndexFile;
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

impl Session {
    pub fn new(description: Option<String>) -> ReplayResult<Self> {
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

    fn load_from_files<T: DeserializeOwned>(session_id: &str) -> ReplayResult<T> {
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

    pub fn load_session_by_index(index: u32) -> ReplayResult<Self> {
        let session_id = SessionIndexFile::get_session_id(index)?;
        Session::load_from_files(&session_id)
    }

    pub fn load_last_session() -> ReplayResult<Self> {
        Self::load_session_by_index(0)
    }

    pub fn load_metadata_by_index(index: &str) -> ReplayResult<MetaData> {
        Session::load_from_files(index)
    }

    pub fn save_session(&self, compress: bool) -> ReplayResult<()> {
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

    pub fn remove_session_by_index(index: u32) -> ReplayResult<()> {
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

    pub fn remove_last_session() -> ReplayResult<()> {
        Self::remove_session_by_index(0)
    }

    pub fn iter_commands(&self) -> impl Iterator<Item = &str> {
        // We use impl Iterator to not have to declare RecordedCommand public
        self.commands.iter().map(|s| s.as_str())
    }
    pub fn get_session_path(id: &str, extension: &str) -> PathBuf {
        paths::session_dir().join(format!("{}.{}", id, extension))
    }

    pub fn get_all_session_metadata() -> ReplayResult<impl Iterator<Item = ReplayResult<MetaData>>>
    {
        Ok(
            SessionIndexFile::iter_session_ids_rev()?.map(|index| -> ReplayResult<MetaData> {
                let index = index?;
                Session::load_metadata_by_index(&index)
            }),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::errors::ReplayError;
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
        assert_eq!(SessionIndexFile::get_session_id(0).unwrap(), session_1.id);

        let session_2 = Session::new(Some("test session 2".into())).unwrap();
        session_2.save_session(true).unwrap();
        assert_eq!(SessionIndexFile::get_session_id(0).unwrap(), session_2.id);
        assert_eq!(
            SessionIndexFile::remove_session_id(0).unwrap(),
            session_2.id
        );
        assert_eq!(
            SessionIndexFile::remove_session_id(0).unwrap(),
            session_1.id
        );

        // Test popping and peeking from empty index file returns error
        assert!(matches!(
            SessionIndexFile::remove_session_id(0),
            Err(ReplayError::SessionError(_))
        ));
        assert!(matches!(
            SessionIndexFile::get_session_id(0),
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
        Session::remove_session_by_index(1).unwrap();
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
        Session::remove_session_by_index(0).unwrap();
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
