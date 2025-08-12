use super::RunnableCommand;
use crate::args::validate_session_description;
use crate::errors::ReplayError;
use crate::session::{Session, TEST_ID};
use clap::Args;
use crossterm::terminal;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread;

#[derive(Args, PartialEq, Eq, Debug)]
pub struct RecordCommand {
    #[arg(value_parser=validate_session_description)]
    session_description: Option<String>,
}

impl RunnableCommand for RecordCommand {
    fn run(&self) -> Result<(), ReplayError> {
        let mut reader = std::io::stdin();
        self.run_internal(&mut reader)
    }
}

impl RecordCommand {
    fn run_internal<R: Read>(&self, input_reader: &mut R) -> Result<(), ReplayError> {
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
        let mut buf = [0u8; 1024];
        let mut cmd_raw: Vec<u8> = Vec::new();
        let mut session = Session::new(self.session_description.clone()); // TODO : use .take() be self needs to be mut

        loop {
            if let Some(_) = child.try_wait()? {
                // replay message
                break;
            }
            match input_reader.read(&mut buf)? {
                0 => break,
                n => {
                    master_writer.write_all(&buf)?;
                    cmd_raw.extend_from_slice(&buf[..n]);
                    if cmd_raw.ends_with(b"\r") {
                        // If the command ends with a newline, we consider it complete.
                        session.add_command(cmd_raw.clone());
                        cmd_raw.clear(); // Reset for the next command
                    }
                }
            }
        }
        terminal::disable_raw_mode()?;
        // We don't want the program to panic at any moment since we catch error in the main program and disable_raw mode here
        output_reader.join().map_err(|paylod| {
            ReplayError::ThreadPanic(format!("`output_reader` with \n {:?}", paylod))
        })??;

        session.save_session()?;

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
        #[cfg(not(test))]
        {
            std::io::stdout().write_all(&buffer[..n])?;
            std::io::stdout().flush()?;
        }
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
#[cfg(test)]
struct RawModeReader {
    data: Vec<u8>,
    pos: usize,
}
#[cfg(test)]
impl RawModeReader {
    fn new(input: &[u8]) -> Self {
        Self {
            data: input.to_vec(),
            pos: 0,
        }
    }
}

#[cfg(test)]
impl std::io::Read for RawModeReader {
    // The following function simulate a raw mode reader reading 1 bytes from the input data at one time
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.data.len() {
            return Ok(0);
        }
        buf[0] = self.data[self.pos];
        self.pos += 1;
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_command_creates_valid_json_sessions() {
        let cmd1 = RecordCommand::new(None);
        let mut reader1 = RawModeReader::new(b"ls\recho test\rexit\r");
        cmd1.run_internal(&mut reader1).unwrap();

        let file_path = String::from(Session::get_session_path(TEST_ID));
        let content = std::fs::read_to_string(&file_path).unwrap();

        let session: Session = serde_json::from_str(&content)
            .expect("The json structure doesn't correspond to the expected session format");

        let mut command_iter = session.iter_commands();

        assert!(command_iter.next().unwrap() == "ls\r");

        assert!(command_iter.next().unwrap() == "echo test\r");

        assert!(command_iter.next().unwrap() == "exit\r");

        assert!(session.description.is_none());
    }
}
