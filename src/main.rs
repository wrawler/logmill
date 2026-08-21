use clap::Parser;
use std::io::{self, BufRead};
use std::sync::mpsc;

// Import our new modules
use logmill::cli::Args; 
use logmill::{archiver, writer}; 

fn main() {
    let args = Args::parse();

    ctrlc::set_handler(move || {
        println!("\n[MAIN] Ctrl+C caught! Waiting for pipe to close and flushing memory...");
    }).expect("Error setting Ctrl-C handler");

    // Create our communication channels
    let (tx_logs, rx_logs) = mpsc::sync_channel(1024);
    let (tx_archive, rx_archive) = mpsc::channel::<String>();

    // Spawn the background threads using our modules
    let archiver_handle = archiver::spawn(rx_archive, args.archive_dir);
    let writer_handle = writer::spawn(rx_logs, tx_archive, args.file, args.max_size);

    // Main thread: read stdin and push to the writer
    let stdin = io::stdin();
    let handle = stdin.lock();

    for line in handle.lines() {
        match line {
            Ok(text) => {
                if tx_logs.send(text).is_err() {
                    break;
                }
            }
            Err(e) => eprintln!("[MAIN] Error reading line: {}", e),
        }
    }

    drop(tx_logs); 
    writer_handle.join().unwrap();
    archiver_handle.join().unwrap();
}