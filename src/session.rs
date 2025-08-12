use crate::errors::ReplayError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;

#[derive(Default, Serialize, Deserialize)]
pub struct Session {
    pub description: Option<String>,
    pub id: String,
    timestamp: chrono::DateTime<Utc>,
    user: String,
    commands: Vec<String>,
}

#[cfg(test)]
pub const TEST_ID: &str = "test_session";

impl Session {
    pub fn new(description: Option<String>) -> Self {
        let user = whoami::username();
        let timestamp = Utc::now();
        Self {
            commands: Vec::new(),
            id: Self::generate_id(&description, &timestamp, &user),
            description: description,
            timestamp,
            user,
        }
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
        description: &Option<String>,
        timestamp: &chrono::DateTime<Utc>,
        user: &str,
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
        todo!("Use DEFAULT_SESSION_PATH to load session by its name, and return SessionError")
    }

    pub fn load_last_session() -> Result<Self, ReplayError> {
        todo!("load last session");
    }

    pub fn save_session(&self) -> Result<(), ReplayError> {
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(Self::get_session_path(&self.id), json)?;
        Ok(())
    }

    pub fn iter_commands(&self) -> impl Iterator<Item = &String> + '_ {
        // We use impl Iterator to not have to declare RecordedCommand public
        self.commands.iter()
    }

    pub fn get_session_path(id: &str) -> String {
        format!(
            "{}/{}.json",
            env::var("HOME").unwrap_or_else(|_| String::from("/home/user/.replay/sessions")),
            id
        )
    }
}
