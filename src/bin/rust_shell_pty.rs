use anyhow::Error;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{BufReader, Read, Write};
use std::sync::mpsc::channel;
use std::thread;

const PROMPT: &str = "<<READY>> ";

fn main() -> Result<(), Error> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    let mut cmd = CommandBuilder::new("/bin/sh");
    cmd.args(&["-i"]);
    cmd.env("PS1", PROMPT);

    let mut child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let writer = pair.master.take_writer()?;
    let mut reader = BufReader::new(pair.master.try_clone_reader()?);

    let (tx, rx) = channel::<String>();
    let (ready_tx, ready_rx) = channel::<()>();

    thread::spawn(move || {
        let mut buffer = String::new();
        let mut chunk = [0u8; 1024];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.push_str(&String::from_utf8_lossy(&chunk[..n]));
                    while let Some(idx) = buffer.find(PROMPT) {
                        let mut part = buffer[..idx].to_string();
                        if let Some(pos) = part.find('\n') {
                            part = part[pos + 1..].to_string();
                        }
                        print!("{}", part);

                        buffer = buffer[idx + PROMPT.len()..].to_string();

                        let _ = ready_tx.send(()).unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from PTY: {}", e);
                    break;
                }
            }
        }
    });

    let writer_handle = thread::spawn(move || handle_input_stream(rx, writer));

    println!("You can now type commands for Bash (type 'exit' to quit):");

    'rust_stdin: loop {
        while let Ok(_) = ready_rx.recv() {
            print!("@replay-shell > "); // TODO probably print an interactive stdin prefix (depending on current session for example)
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if input.trim() == "exit" {
                break 'rust_stdin;
            }
            tx.send(input).unwrap();
        }
    }
    // STOP THE WRITING
    // We drop tx in case we exit to ensure the 'input in rx.iter()' stop
    drop(tx);

    // Because the 'input in rx.iter()' stopped, the writer_handle thread is finished
    writer_handle.join().unwrap();

    // STOP THE SHELL PROCESS
    println!("Waiting for Bash to exit…");
    let status = child.wait()?;
    println!("Bash exited with status: {:?}", status);
    Ok(())
}

fn handle_input_stream(rx: std::sync::mpsc::Receiver<String>, mut writer: Box<dyn Write + Send>) {
    for input in rx.iter() {
        if writer.write_all(input.as_bytes()).is_err() {
            eprintln!("Error writing to PTY");
            break;
        }
        let _ = writer.flush();
    }
}
