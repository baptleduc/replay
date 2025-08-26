use crate::char_buffer::CharBuffer;
use crate::errors::ReplayError;
use crate::session::Session;
use crossterm::terminal;
use portable_pty::{Child, CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread::{self, JoinHandle};
use std::time::Duration;

type Reader = Box<dyn Read + Send>;
type Writer = Box<dyn Write + Send>;
type ChildProc = Box<dyn Child + Send + Sync>;

pub fn run_internal<R: Read, W: Write + Send + 'static>(
    user_input: R,                       // input from user (stdin, pipe…)
    user_output: W,                      // output to user (stdout, file…)
    record_user_input: bool,             // enable recording of typed commands
    session_description: Option<String>, // optional session description
    no_compression: bool,                // disable compression
    delay: u64,                          // add some delay between commands executions
) -> Result<(), ReplayError> {
    terminal::enable_raw_mode()?;

    let (pty_stdout, pty_stdin, child) = spawn_shell()?;

    // Thread to read from the PTY and send data by user_output.
    let output_reader = thread::spawn(move || read_from_pty(pty_stdout, user_output));

    let exit_msg = handle_user_input(
        user_input,
        pty_stdin,
        child,
        record_user_input,
        session_description,
        no_compression,
        delay,
    )?;
    terminal::disable_raw_mode()?;
    join_output_thread(output_reader)?;

    println!("{}", exit_msg);
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
    mut child: ChildProc,
    record_input: bool,
    session_description: Option<String>,
    no_compression: bool,
    delay: u64,
) -> Result<String, ReplayError> {
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

        let n = user_input.read(&mut buf)?;
        if n == 0 {
            break; // EOF
        } else if n != 1 {
            unreachable!("Unexpected read size, should be 1 in terminal raw mode!");
        }

        let c = buf[0];

        // Handle input locally
        match c {
            // Backspace
            b'\x7F' => {
                char_buffer.pop_char();
            }
            // Ctrl+W
            b'\x17' => {
                char_buffer.pop_word();
            }
            // Ctrl+C
            b'\x03' => {
                if let Some(sess) = session.as_mut() {
                    sess.remove_last_command();
                }
                char_buffer.clear();
            }
            // Enter key
            b'\r' => {
                // Normal line submission
                char_buffer.push_char(b'\r');
                if let Some(sess) = session.as_mut() {
                    sess.add_command(char_buffer.get_buf().to_vec());
                }

                // q + enter : quit without saving the session
                if char_buffer.get_buf() == b"q\r" {
                    child.kill()?;
                    session = None; // Don't save session
                    break;
                }

                // Exit
                if char_buffer.get_buf() == b"exit\r" {
                    // We drop `pty_stdin` instead of `child` to ensure it close properly
                    drop(pty_stdin);
                    break;
                }
                char_buffer.clear();
            }

            _ => {
                char_buffer.push_char(c);
            } // Any other character
        }

        // Send input to PTY
        pty_stdin.write_all(&buf)?;
        pty_stdin.flush()?;
        if c == b'\r' && delay > 0 {
            std::thread::sleep(Duration::from_millis(delay));
        }
    }

    if let Some(sess) = session {
        sess.save_session(!no_compression)?;
        return Ok(String::from("Session saved"));
    }

    Ok(String::from("\nNo session saved"))
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
    use crate::paths::clear_replay_dir;
    use serial_test::serial;
    use std::io::sink;

    /// Helper to run a fake session and return the list of recorded commands.
    fn run_and_get_commands(input: &[u8]) -> Vec<String> {
        let reader = RawModeReader::new(input);
        let _ = run_internal(reader, sink(), true, None, false, 0);

        Session::load_last_session()
            .map(|sess| {
                sess.iter_commands()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    #[test]
    #[serial]
    fn record_commands_with_ctrl_c() {
        clear_replay_dir().unwrap();
        let cmds = run_and_get_commands(b"echo test_ctrl_c\rsleep 5\r\x03exit\r");

        assert_eq!(
            cmds,
            vec!["echo test_ctrl_c\r", "exit\r"],
            "Expected only echo and exit to be saved when Ctrl+C is used"
        );
    }

    #[test]
    #[serial]
    fn record_commands_with_q_enter() {
        clear_replay_dir().unwrap();
        let cmds = run_and_get_commands(b"echo q\rq\r");

        assert!(
            cmds.is_empty(),
            "No session should be saved when quitting with q+Enter"
        );
    }

    #[test]
    #[serial]
    fn record_commands_with_ctrl_w() {
        clear_replay_dir().unwrap();
        let cmds = run_and_get_commands(b"echo 1 2\x17\rexit\r");

        assert_eq!(
            cmds,
            vec!["echo 1 \r", "exit\r"],
            "Ctrl+W should delete the last word before saving"
        );
    }

    #[test]
    #[serial]
    fn record_commands_with_all_control_chars() {
        clear_replay_dir().unwrap();
        let cmds = run_and_get_commands(b"ls\recho\x7Fo test\x17test\rexit\r");

        assert_eq!(
            cmds,
            vec!["ls\r", "echo test\r", "exit\r"],
            "Combination of Backspace + Ctrl+W should still produce valid commands"
        );

        let session = Session::load_last_session().unwrap();
        assert!(
            session.description.is_none(),
            "Session description should remain None by default"
        );
    }
}
