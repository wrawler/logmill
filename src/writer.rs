use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

fn open_log_file(path: &str) -> File {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Failed to open or create the log file")
}

/// Spawns a background thread to write logs and rotate them when max_size is hit.
pub fn spawn(
    rx_logs: Receiver<String>, 
    tx_archive: Sender<String>, 
    log_path: String, 
    max_size: usize
) -> JoinHandle<()> {
    
    thread::spawn(move || {
        let mut file = open_log_file(&log_path);
        let mut current_size = file.metadata().unwrap().len() as usize;
        let mut rotation_count = 1;

        println!("[WRITER] Target: {}, Max Size: {} bytes", log_path, max_size);

        for received_line in rx_logs {
            let bytes_to_write = received_line.len() + 1;

            if current_size + bytes_to_write > max_size {
                file.sync_all().unwrap(); 

                let rotated_name = format!("{}.{}", log_path, rotation_count);
                fs::rename(&log_path, &rotated_name).expect("Failed to rename file");
                
                rotation_count += 1;
                file = open_log_file(&log_path);
                current_size = 0;

                tx_archive.send(rotated_name).unwrap();
            }

            if let Err(e) = writeln!(file, "{}", received_line) {
                eprintln!("[WRITER] Error writing to disk: {}", e);
            } else {
                current_size += bytes_to_write;
            }
        }
        
        drop(tx_archive); // Signal archiver to close
        println!("[WRITER] Stream ended. Shutting down.");
    })
}