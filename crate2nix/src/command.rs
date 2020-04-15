//! Utilities for dealing with spawned commands.

use anyhow::{bail, format_err, Error};
use std::process::{Child, Stdio};
use std::thread;
use std::{
    io::{BufRead, Read},
    io::{BufReader, Cursor},
    sync::mpsc,
};

/// Runs the given command with output capturing.
///
/// The output will be printed indented if and only if the command does not
/// return succesfully.
pub fn run(caption: &str, command: &mut std::process::Command) -> Result<(), Error> {
    eprint!("{}: ", caption);

    let mut spawned: Child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format_err!("while spawning {:?}: {}", command, e))?;

    let (sender, receiver) = mpsc::channel();

    pass_through(spawned.stdout.take().expect("stdout"), sender.clone());
    pass_through(spawned.stderr.take().expect("stderr"), sender);

    let mut out = Vec::<u8>::new();
    while let Ok(buf) = receiver.recv() {
        out.extend(buf.iter());
    }

    let status = spawned
        .wait()
        .map_err(|e| format_err!("while waiting for the {:?} to finish: {}", command, e))?;

    if status.success() {
        eprintln!("done.");
        return Ok(());
    }

    eprintln!();
    eprintln!("  {:?}", command);
    let line_reader = BufReader::new(Cursor::new(out));
    for line in line_reader.lines() {
        println!(
            "  {}",
            line.map_err(|e| format_err!("while processing output lines: {}", e))?
        );
    }

    bail!(
        "{:?}\n=> exited with: {}",
        command,
        status.code().unwrap_or(-1)
    );
}

fn pass_through(mut read: impl Read + Send + 'static, sender: mpsc::Sender<Vec<u8>>) {
    thread::spawn(move || {
        let mut buf = [0; 4096];
        while let Ok(n) = read.read(&mut buf) {
            if n == 0 {
                break;
            }
            if sender.send(Vec::from(&buf[..n])).is_err() {
                break;
            }
        }
    });
}
