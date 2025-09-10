use crate::errors::{ReplayError, ReplayResult};
use crate::paths;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const INDEX_SIZE: u64 = 64;
pub struct SessionIndexFile;

struct RevIndexIter {
    file: File,
    index_size: u64,
    current: u64,
}

impl RevIndexIter {
    pub fn new(file: File, index_size: u64) -> ReplayResult<Self> {
        let metadata = file.metadata()?;
        let total_records = metadata.len() / index_size;
        Ok(Self {
            file,
            index_size,
            current: total_records,
        })
    }
}

impl Iterator for RevIndexIter {
    type Item = ReplayResult<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == 0 {
            return None;
        }
        self.current -= 1;
        let pos = self.current * self.index_size;
        let result = (|| -> ReplayResult<String> {
            self.file.seek(SeekFrom::Start(pos))?;
            let mut buf = vec![0u8; self.index_size as usize];
            self.file.read_exact(&mut buf)?;
            Ok(String::from_utf8_lossy(&buf).to_string())
        })();

        Some(result)
    }
}

impl SessionIndexFile {
    pub(super) fn get_path() -> PathBuf {
        paths::replay_dir().join("session_idx")
    }

    fn open_file() -> ReplayResult<std::fs::File> {
        Ok(std::fs::OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(Self::get_path())?)
    }

    pub fn push_session(session_id: &str) -> ReplayResult<()> {
        let mut file = Self::open_file()?;
        write!(file, "{}", session_id)?;
        Ok(())
    }

    fn get_id_offset_by_index(n: u32) -> ReplayResult<u64> {
        let file = Self::open_file()?;
        let file_size = file.metadata()?.len();
        if file_size == 0 {
            return Err(ReplayError::SessionError("No replay entries found".into()));
        }

        let total_lines = file_size / INDEX_SIZE;
        if n as u64 >= total_lines {
            return Err(ReplayError::SessionError(
                "Replay index out of range".into(),
            ));
        }

        Ok(file_size - (n as u64 + 1) * INDEX_SIZE)
    }

    fn read_id_at(offset: u64) -> Result<String, ReplayError> {
        let mut file = Self::open_file()?;
        file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; INDEX_SIZE as usize];
        file.read_exact(&mut buf)?;

        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    /// Get the nth session id and remove it from the file
    pub fn remove_session_id(n: u32) -> ReplayResult<String> {
        let mut file = Self::open_file()?;
        let id_offset = Self::get_id_offset_by_index(n)?;

        let session_id = Self::read_id_at(id_offset)?;

        // Calculate the position of next id
        let next_id_offset = id_offset + INDEX_SIZE;

        // Read the end of the file after this line
        let mut rest = Vec::new();
        file.seek(SeekFrom::Start(next_id_offset))?;
        file.read_to_end(&mut rest)?;

        // Truncate and rewrite the rest
        file.set_len(id_offset)?;
        file.seek(SeekFrom::Start(id_offset))?;
        file.write_all(&rest)?;
        file.flush()?;

        Ok(session_id)
    }

    pub fn get_session_id(index: u32) -> ReplayResult<String> {
        let line_offset = Self::get_id_offset_by_index(index)?;
        Self::read_id_at(line_offset)
    }
    pub fn iter_session_ids_rev() -> ReplayResult<impl Iterator<Item = ReplayResult<String>>> {
        let file = SessionIndexFile::open_file()?;
        let iter = RevIndexIter::new(file, INDEX_SIZE)?;
        Ok(iter)
    }
}
