use crate::DEFAULT_SESSION_PATH;
use std::io;

#[derive(Default)]
pub struct Session {
    pub name: String,
    pub description: Option<String>,
    commands: Vec<String>,
}

impl Session {
    pub fn new(session_name: String, description: Option<String>) -> Self {
        Self {
            name: session_name,
            description,
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: String) {
        self.commands.push(command);
    }

    pub fn to_json(&self) {
        todo!("implement json structure");
    }

    pub fn load_session(session_name: &str) -> Result<Self, &'static str> {
        todo!("Use DEFAULT_SESSION_PATH to load session by its name")
    }

    pub fn load_last_session() -> Result<Self, &'static str> {
        todo!("load last session");
    }

    pub fn iter_commands(&self) -> impl Iterator<Item = &String> {
        self.commands.iter()
    }
}
