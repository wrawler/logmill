use clap::Parser;
use std::io::{self, BufRead};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

// Import our new modules
use logmill::cli::Args;
use logmill::{archiver, writer};

fn main() {
    if let Err(error) = run() {
        eprintln!("[MAIN] {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse();
    let shutdown = Arc::new(AtomicBool::new(false));
    let signal_shutdown = Arc::clone(&shutdown);

    ctrlc::set_handler(move || {
        signal_shutdown.store(true, Ordering::SeqCst);
    })
    .map_err(|e| format!("failed to install signal handler: {e}"))?;

    // Create our communication channels
    let (tx_logs, rx_logs) = mpsc::sync_channel(1024);
    let (tx_archive, rx_archive) = mpsc::channel::<String>();
    let (tx_input, rx_input) = mpsc::channel::<io::Result<String>>();

    let archiver_handle = archiver::spawn(rx_archive, args.archive_dir);
    let writer_handle = writer::spawn(rx_logs, tx_archive, args.file, args.max_size);

    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if tx_input.send(line).is_err() {
                break;
            }
        }
    });

    while !shutdown.load(Ordering::SeqCst) {
        match rx_input.recv_timeout(Duration::from_millis(100)) {
            Ok(text) => match text {
                Ok(text) => {
                    if tx_logs.send(text).is_err() {
                        break;
                    }
                }
                Err(error) => eprintln!("[MAIN] Error reading line: {error}"),
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(rx_input);
    drop(tx_logs);
    writer_handle
        .join()
        .map_err(|_| "writer thread panicked".to_string())??;
    archiver_handle
        .join()
        .map_err(|_| "archiver thread panicked".to_string())??;
    Ok(())
}
