use crossterm::terminal;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread;

fn main() -> Result<(), anyhow::Error> {
    terminal::enable_raw_mode()?;
    let pty_system = NativePtySystem::default();

    // Open the PTY with specified size.
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    // Set up the command to launch Bash.
    let cmd = CommandBuilder::new("/bin/bash");
    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave); // No longer used

    // Set up master reader and writer
    let master_reader = pair.master.try_clone_reader().unwrap();
    let mut master_writer = pair.master.take_writer().unwrap();

    // Thread to read from the PTY and send data to the main thread.
    let _ = thread::spawn(move || {
        read_from_pty(master_reader);
    });

    // Main thread sends user input to the writer thread.
    let mut stdin = std::io::stdin();
    let mut buf = [0u8; 1];

    loop {
        if let Some(_) = child.try_wait()? {
            // replay message
            break;
        }
        match stdin.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                if master_writer.write_all(&buf).is_err() {
                    eprintln!("Error writing to PTY");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error during stdin reading: {e}");
                break;
            }
        }
    }
    terminal::disable_raw_mode()?;
    Ok(())
}

fn read_from_pty(mut reader: Box<dyn Send + Read>) {
    let mut buffer = [0u8; 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(n) => {
                std::io::stdout().write_all(&buffer[..n]).unwrap();
                std::io::stdout().flush().unwrap();
            }
            Err(e) => {
                eprintln!("Error reading from PTY: {}", e);
                break;
            }
        }
    }
}
