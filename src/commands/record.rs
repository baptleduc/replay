use super::RunnableCommand;
use crate::args::validate_session_description;
use crate::errors::ReplayError;
use crate::session::Session;
use chrono::Utc;
use clap::Args;
use crossterm::terminal;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::thread;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct RecordCommand {
    #[arg(value_parser=validate_session_description)]
    session_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecordedCommand {
    cmd_str: String,
    cmd_raw: Vec<u8>,
    timestamp: chrono::DateTime<Utc>,
    user: String,
}

impl RecordedCommand {
    pub fn new(cmd_raw: Vec<u8>) -> Self {
        let cmd_str = String::from_utf8_lossy(&cmd_raw).to_string();
        let timestamp = chrono::Utc::now();
        let user = whoami::username();
        RecordedCommand {
            cmd_str,
            cmd_raw,
            timestamp,
            user,
        }
    }
    pub fn to_json_file(&self) -> Result<(), ReplayError> {
        let json_data =
            serde_json::to_string(self).map_err(|e| ReplayError::ExportError(e.to_string()))?;
        std::fs::write(Session::get_default_session_path(), json_data)
            .map_err(|e| ReplayError::ExportError(e.to_string()))?;
        Ok(())
    }
}

impl RunnableCommand for RecordCommand {
    fn run(&self) -> Result<(), ReplayError> {
        terminal::enable_raw_mode()?;
        let pty_system = NativePtySystem::default();

        // Open the PTY with specified size.
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Set up the command to launch Bash.
        let cmd = CommandBuilder::new("/bin/bash");
        let mut child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave); // No longer used

        // Set up master reader and writer
        let master_reader = pair.master.try_clone_reader()?;
        let mut master_writer = pair.master.take_writer()?;

        // Thread to read from the PTY and send data to the main thread.
        let output_reader = thread::spawn(move || read_from_pty(master_reader));

        // Main thread sends user input to bash stdin
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 1024];
        let mut cmd_raw: Vec<u8> = Vec::new();

        loop {
            if let Some(_) = child.try_wait()? {
                // replay message
                break;
            }
            match stdin.read(&mut buf)? {
                0 => break,
                n => {
                    master_writer.write_all(&buf)?;
                    cmd_raw.extend_from_slice(&buf[..n]);

                    if cmd_raw.ends_with(b"\r") {
                        // If the command ends with a newline, we consider it complete.
                        let recorded_command = RecordedCommand::new(cmd_raw);
                        recorded_command.to_json_file()?;
                        cmd_raw = Vec::new(); // Reset for the next command
                    }
                }
            }
        }
        terminal::disable_raw_mode()?;
        // We don't want the program to panic at any moment since we catch error in the main program and disable_raw mode here
        output_reader.join().map_err(|paylod| {
            ReplayError::ThreadPanic(format!("`output_reader` with \n {:?}", paylod))
        })??;
        Ok(())
    }
}

fn read_from_pty(mut reader: Box<dyn Send + Read>) -> Result<(), ReplayError> {
    let mut buffer = [0u8; 1024];
    loop {
        let n = reader.read(&mut buffer)?;
        // EOF, if the child entry is closed, we want the thread to exit
        if n == 0 {
            break;
        }
        std::io::stdout().write_all(&buffer[..n])?;
        std::io::stdout().flush()?;
    }
    Ok(())
}

#[cfg(test)]
impl RecordCommand {
    pub fn new(desc: Option<String>) -> Self {
        RecordCommand {
            session_description: desc,
        }
    }
}
