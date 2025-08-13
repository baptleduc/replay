use crate::errors::ReplayError;
use crate::session::Session;
use crossterm::terminal;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread;

pub fn run_internal<R: Read, W: Write + Send + 'static>(
    mut user_input: R,            // input from user (stdin, pipe…)
    user_output: W,               // output to user (stdout, file…)
    record_user_input: bool,      // enable recording of typed commands
    session_info: Option<String>, // optional session description
) -> Result<(), ReplayError> {
    terminal::enable_raw_mode()?;
    let pty_system = NativePtySystem::default();

    // Open a pseudo-terminal
    let pty_pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Spawn bash inside PTY
    let bash_cmd = CommandBuilder::new("/bin/bash");
    let mut bash_process = pty_pair.slave.spawn_command(bash_cmd)?;
    drop(pty_pair.slave); // not needed anymore

    // PTY handles for I/O
    let pty_stdout = pty_pair.master.try_clone_reader()?; // bash → user
    let mut pty_stdin = pty_pair.master.take_writer()?; // user → bash

    // Thread: forward PTY output to user output
    let output_thread = thread::spawn(move || read_from_pty(pty_stdout, user_output));

    // Buffers for input and command recording
    let mut input_buffer = [0u8; 1024];
    let mut current_command: Vec<u8> = Vec::new();

    // Initialize session recording if enabled
    let mut session: Option<Session> = if record_user_input {
        Some(Session::new(session_info)?)
    } else {
        None
    };

    loop {
        // Exit if bash process ended
        if let Some(_) = bash_process.try_wait()? {
            break;
        }

        match user_input.read(&mut input_buffer)? {
            0 => break, // no more input
            n => {
                // Forward user input to bash (slave)
                pty_stdin.write_all(&input_buffer[..n])?;

                // Optionally record commands
                if record_user_input {
                    current_command.extend_from_slice(&input_buffer[..n]);
                    if current_command.ends_with(b"\r") {
                        if let Some(sess) = session.as_mut() {
                            sess.add_command(current_command.clone());
                        }
                        current_command.clear();
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode()?;

    // Wait for output thread to finish
    output_thread.join().map_err(|payload| {
        ReplayError::ThreadPanic(format!("`output_thread` panicked with \n {:?}", payload))
    })??;

    // Save session if recording enabled
    if record_user_input {
        if let Some(sess) = session.as_mut() {
            sess.save_session()?;
        }
    }

    Ok(())
}

fn read_from_pty<R: Read + Send, W: Write + Send>(
    mut pty_output: R,  // PTY → bash output
    mut user_output: W, // user-visible output
) -> Result<(), ReplayError> {
    let mut buffer = [0u8; 1024];
    loop {
        let n = pty_output.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        user_output.write_all(&buffer[..n])?;
        user_output.flush()?;
    }
    Ok(())
}

pub struct RawModeReader {
    data: Vec<u8>,
    pos: usize,
}
impl RawModeReader {
    pub fn new(input: &[u8]) -> Self {
        Self {
            data: input.to_vec(),
            pos: 0,
        }
    }
}
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
mod test {
    use super::*;
    use std::fs;
    use std::io::sink;
    #[test]
    fn record_creates_valid_json_sessions() {
        let mut reader1 = RawModeReader::new(b"ls\recho test\rexit\r");
        run_internal(&mut reader1, Box::new(sink()), true, None).unwrap();
        let file_path = Session::get_session_path("test_session");
        let content = fs::read_to_string(&file_path).unwrap();
        let session: Session = serde_json::from_str(&content)
            .expect("The json structure doesn't correspond to the expected session format");

        let mut command_iter = session.iter_commands();
        assert_eq!(command_iter.next().unwrap(), "ls\r");
        assert_eq!(command_iter.next().unwrap(), "echo test\r");
        assert_eq!(command_iter.next().unwrap(), "exit\r");
        assert!(session.description.is_none());
        fs::remove_file(file_path).unwrap();
    }
}
