use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

/// Spawns a background thread to compress rotated log files.
pub fn spawn(rx_archive: Receiver<String>, archive_dir: String) -> JoinHandle<()> {
    thread::spawn(move || {
        // 1. Ensure the target directory actually exists before we start
        if let Err(e) = fs::create_dir_all(&archive_dir) {
            eprintln!("[ARCHIVER] Failed to create archive directory: {}", e);
            return;
        }

        println!("[ARCHIVER] Ready. Saving archives to: {}", archive_dir);
        
        for file_to_archive in rx_archive {
            // 2. Safely extract the filename from the path
            // e.g., turns "/var/log/app.log.1" into "app.log.1"
            let source_path = Path::new(&file_to_archive);
            let file_name = source_path.file_name().unwrap().to_string_lossy();
            
            // 3. Join the target directory with the new filename safely
            let archive_path = Path::new(&archive_dir)
                .join(format!("{}.tar.gz", file_name));
            
            let archive_name = archive_path.to_string_lossy();

            println!("[ARCHIVER] Compressing {} -> {}", file_to_archive, archive_name);

            // Execute the standard Linux 'tar' command
            let status = Command::new("tar")
                .args(["-czf", archive_name.as_ref(), &file_to_archive])
                .status()
                .expect("Failed to execute tar command");

            if status.success() {
                fs::remove_file(&file_to_archive).unwrap();
            } else {
                eprintln!("[ARCHIVER] Critical Error: Tar command failed!");
            }
        }
        println!("[ARCHIVER] Shutting down.");
    })
}