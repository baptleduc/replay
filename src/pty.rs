use crate::char_buffer::CharBuffer;
use crate::errors::ReplayError;
use crate::session::Session;
use crossterm::terminal;
use portable_pty::{Child, CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread::{self, JoinHandle};

type Reader = Box<dyn Read + Send>;
type Writer = Box<dyn Write + Send>;
type ChildProc = Box<dyn Child + Send + Sync>;

pub fn run_internal<R: Read, W: Write + Send + 'static>(
    user_input: R,                       // input from user (stdin, pipe…)
    user_output: W,                      // output to user (stdout, file…)
    record_user_input: bool,             // enable recording of typed commands
    session_description: Option<String>, // optional session description
) -> Result<(), ReplayError> {
    terminal::enable_raw_mode()?;

    let (pty_stdout, pty_stdin, child) = spawn_shell()?;

    // Thread to read from the PTY and send data by user_output.
    let output_reader = thread::spawn(move || read_from_pty(pty_stdout, user_output));

    handle_user_input(
        user_input,
        pty_stdin,
        record_user_input,
        session_description,
        child,
    )?;
    terminal::disable_raw_mode()?;
    join_output_thread(output_reader)?;
    Ok(())
}

fn spawn_shell() -> Result<(Reader, Writer, ChildProc), ReplayError> {
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
    let bash_process = pty_pair.slave.spawn_command(bash_cmd)?;
    drop(pty_pair.slave); // not needed anymore

    // PTY handles for I/O
    let pty_stdout = pty_pair.master.try_clone_reader()?; // bash → user
    let pty_stdin = pty_pair.master.take_writer()?; // user → bash
    Ok((pty_stdout, pty_stdin, bash_process))
}

// Precondition: Terminal is in raw mode
fn handle_user_input<R: Read, W: Write>(
    mut user_input: R,
    mut pty_stdin: W,
    record_input: bool,
    session_description: Option<String>,
    mut child: ChildProc,
) -> Result<(), ReplayError> {
    // Main thread sends user input to bash stdin
    let mut buf = [0u8; 1]; // We only read one byte in raw mode
    let mut char_buffer = CharBuffer::new();
    let mut session: Option<Session> = if record_input {
        Some(Session::new(session_description)?)
    } else {
        None
    };

    loop {
        if child.try_wait()?.is_some() {
            // Check if the child process has exited
            break;
        }

        match user_input.read(&mut buf)? {
            0 => break, // EOF
            1 => {
                // Send to PTY
                pty_stdin.write_all(&buf)?;
                pty_stdin.flush()?;
                if record_input {
                    // Char deletion handling (Backspace)
                    if buf[0] == b'\x7F' {
                        char_buffer.pop_char();
                        continue; // Not recording backspace
                    }

                    // Word deletion handling (Ctrl+W)
                    if buf[0] == b'\x17' {
                        char_buffer.pop_word();
                        continue; // Not recording Ctrl-W
                    }

                    // Char addition handling
                    char_buffer.push_char(buf[0]);

                    // End of line handling
                    if char_buffer.peek_char() == Some(&b'\r') {
                        if let Some(sess) = session.as_mut() {
                            sess.add_command(char_buffer.get_buf().to_vec());
                        }
                        char_buffer.clear();
                    }
                }
            }
            _ => {
                unreachable!("Unexpected read size, should be 1 in terminal raw mode !");
            }
        }
    }

    if record_input && session.as_mut().is_some() {
        session.as_mut().unwrap().save_session(false)?; // By default, we compress session files
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

fn join_output_thread(
    output_thread: JoinHandle<Result<(), ReplayError>>,
) -> Result<(), ReplayError> {
    // We don't want the program to panic at any moment since we catch error in the main program
    output_thread
        .join()
        .map_err(|err| ReplayError::ThreadPanic(format!("`user_output` with \n {:?}", err)))??;
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
    use serial_test::serial;
    use std::io::sink;

    // #[test]
    // #[serial]
    // fn record_creates_valid_json_sessions() {
    //     let reader1 = RawModeReader::new(b"ls\recho\x7Fo test\x17test\rexit\r");
    //     run_internal(reader1, Box::new(sink()), true, None).unwrap();
    //     let session = Session::load_last_session().unwrap();
    //     let mut command_iter = session.iter_commands();
    //     assert_eq!(command_iter.next().unwrap(), "ls\r");
    //     assert_eq!(command_iter.next().unwrap(), "echo test\r");
    //     assert_eq!(command_iter.next().unwrap(), "exit\r");
    //     assert!(session.description.is_none());
    //     // TODO: delete the file by calling a future remove_last_session()
    // }
}
