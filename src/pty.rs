use crate::char_buffer::CharBuffer;
use crate::errors::{ReplayError, ReplayResult};
use crate::session::Session;
use crossterm::terminal;
use portable_pty::{Child, CommandBuilder, NativePtySystem, PtySize, PtySystem};
use regex::Regex;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

type Reader = Box<dyn Read + Send>;
type Writer = Box<dyn Write + Send>;
type ChildProc = Box<dyn Child + Send + Sync>;

#[derive(Default)]
pub struct RecordConfig {
    pub record_input: bool,                  // enable recording of typed commands
    pub session_description: Option<String>, // optional session description
    pub no_compression: bool,                // disable compression
}

pub fn run_internal<R: Read, W: Write + Send + 'static>(
    user_input: R,               // input from user (stdin, pipe…)
    user_output: W,              // output to user (stdout, file…)
    record_config: RecordConfig, // input config (recording, description, compression)
) -> ReplayResult<()> {
    terminal::enable_raw_mode()?;
    let (ps1_received_sender, ps1_received_receiver) = mpsc::sync_channel::<()>(1);
    let (command_sent_sender, command_sent_receiver) = mpsc::sync_channel::<()>(1);
    let (mut pty_stdout, mut pty_stdin, child) = spawn_shell()?;
    let ps1 = get_last_ps1_char(&mut pty_stdin, &mut pty_stdout)?;

    // Thread to read from the PTY and send data by user_output.
    let output_reader = thread::spawn(move || {
        read_from_pty(
            pty_stdout,
            user_output,
            ps1_received_sender,
            command_sent_receiver,
            ps1,
        )
    });

    let exit_msg = handle_user_input(
        user_input,
        pty_stdin,
        child,
        ps1_received_receiver,
        command_sent_sender,
        record_config,
    )?;
    terminal::disable_raw_mode()?;
    join_output_thread(output_reader)?;

    if let Some(msg) = exit_msg {
        println!("{}", msg);
    }

    Ok(())
}

fn spawn_shell() -> ReplayResult<(Reader, Writer, ChildProc)> {
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
fn is_env_var_output(line: &str) -> bool {
    let line = line.trim();
    let re_ansi = regex::Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]").unwrap();
    let line = re_ansi.replace_all(line, "");

    line.starts_with("$(") && line.ends_with(")")
}

pub fn get_last_ps1_char(pty_stdin: &mut Writer, pty_stdout: &mut Reader) -> ReplayResult<char> {
    let mut last_output = String::from("$PS1");
    let mut reader: BufReader<&mut Box<dyn Read + Send>> = BufReader::new(pty_stdout);
    let re_non_printable = regex::Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]|\x01|\x02").unwrap();

    loop {
        pty_stdin.write_all(format!("echo \"{}\" \r", last_output).as_bytes())?;
        pty_stdin.flush()?;
        let mut line = String::new();
        loop {
            reader.read_line(&mut line)?;
            let line_cleaned = re_non_printable.replace_all(&line, "");
            let line_trimmed = line_cleaned.trim();
            if !line_trimmed.is_empty() && !line_trimmed.contains("echo") {
                if !is_env_var_output(line_trimmed) {
                    let last_char = line_trimmed.chars().next_back().unwrap();
                    return Ok(last_char);
                }

                last_output = line_trimmed.to_string();
                break;
            }
            line.clear();
        }
    }
}

// Precondition: Terminal is in raw mode
fn handle_user_input<R: Read, W: Write>(
    mut user_input: R,
    mut pty_stdin: W,
    mut child: ChildProc,
    bash_ready_receiver: Receiver<()>,
    command_sent_sender: SyncSender<()>,
    record_config: RecordConfig,
) -> ReplayResult<Option<String>> {
    // Main thread sends user input to bash stdin
    let mut buf = [0u8; 1]; // We only read one byte in raw mode
    let mut char_buffer = CharBuffer::new();
    let exit_re = Regex::new(r"^\s*exit\s*$").unwrap();
    let mut first_init = true;
    let mut session: Option<Session> = if record_config.record_input {
        Some(Session::new(record_config.session_description)?)
    } else {
        None
    };
    loop {
        if child.try_wait()?.is_some() {
            // Check if the child process has exited
            break;
        }
        if first_init {
            bash_ready_receiver.recv().unwrap();
            first_init = false;
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
                if exit_re.is_match(std::str::from_utf8(char_buffer.get_buf())?) {
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

        if buf[0] == b'\r' {
            // We sent a signal to indicate that we need to detect a NEW prompt
            command_sent_sender.send(()).unwrap();

            // We block the main thread
            bash_ready_receiver.recv().unwrap();
        }
    }

    let session_saved = if let Some(sess) = session {
        sess.save_session(!record_config.no_compression)?;
        Some("Session saved".to_string())
    } else if record_config.record_input {
        Some("No session saved".to_string())
    } else {
        None
    };

    Ok(session_saved)
}

fn read_from_pty<R: Read + Send, W: Write + Send>(
    mut pty_output: R,
    mut user_output: W,
    bash_ready_sender: SyncSender<()>,
    command_sent_receiver: Receiver<()>,
    ps1: char,
) -> ReplayResult<()> {
    let mut read_buf = [0u8; 1024];
    let mut ps1_detected: bool = false;
    let re_non_printable =
        Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]|[\x01\x02]|\x1b\][^\x07]*\x07|\x1b\??\d*[hl]").unwrap();

    loop {
        let n = pty_output.read(&mut read_buf)?;
        if n == 0 {
            break; // EOF
        }

        // After the main thread sends a command, reset `ps1_detected` to false.
        // The next detected prompt (ending with `ps1`) will then signal that Bash is ready.
        if command_sent_receiver.try_recv().is_ok() {
            ps1_detected = false;
        };

        user_output.write_all(&read_buf[..n])?;
        user_output.flush()?;

        let tail_vec: &Vec<u8> = &read_buf[..n].to_vec();
        let tail_str = String::from_utf8_lossy(tail_vec);
        let cleaned = re_non_printable
            .replace_all(&tail_str, "")
            .trim()
            .to_string();
        if cleaned.ends_with(&ps1.to_string()) && !ps1_detected {
            let _ = bash_ready_sender.send(());
            ps1_detected = true;
        }
    }

    Ok(())
}

fn join_output_thread(output_thread: JoinHandle<ReplayResult<()>>) -> ReplayResult<()> {
    // We don't want the program to panic at any moment since we catch error in the main program
    output_thread
        .join()
        .map_err(|err| ReplayError::ThreadPanic(format!("`user_output` with \n {:?}", err)))??;
    Ok(())
}

pub struct RawModeReader {
    data: Vec<u8>,
    pos: usize,
    delay: Duration,
}

impl Default for RawModeReader {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            pos: 0,
            delay: Duration::from_millis(0),
        }
    }
}

impl RawModeReader {
    pub fn with_input(input: &[u8]) -> Self {
        Self {
            data: input.to_vec(),
            pos: 0,
            delay: Duration::from_millis(0),
        }
    }

    pub fn with_input_and_delay(input: &[u8], delay: Duration) -> Self {
        Self {
            data: input.to_vec(),
            pos: 0,
            delay,
        }
    }
}
impl std::io::Read for RawModeReader {
    // The following function simulate a raw mode reader reading 1 bytes from the input data at one time
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.data.len() {
            return Ok(0);
        }
        std::thread::sleep(self.delay);
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
        let test_input_config: RecordConfig = RecordConfig {
            record_input: true,
            ..Default::default()
        };
        let reader = RawModeReader::with_input(input);
        let _ = run_internal(reader, sink(), test_input_config);

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

    #[test]
    #[serial]
    fn record_exit_command_only() {
        clear_replay_dir().unwrap();
        let cmds = run_and_get_commands(b"echo exit\r     exit     \r");

        assert_eq!(
            cmds,
            vec!["echo exit\r", "     exit     \r"],
            "Expected echo and exit commands to be saved"
        );
    }
}
