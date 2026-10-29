use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

fn open_log_file(path: &str) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

fn next_rotated_path(log_path: &Path) -> PathBuf {
    let mut rotation_count = 1;
    loop {
        let candidate = PathBuf::from(format!("{}.{}", log_path.display(), rotation_count));
        if !candidate.exists() {
            return candidate;
        }
        rotation_count += 1;
    }
}

/// Spawns a background thread to write logs and rotate them when max_size is hit.
pub fn spawn(
    rx_logs: Receiver<String>,
    tx_archive: Sender<String>,
    log_path: String,
    max_size: usize,
) -> JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let log_path_ref = Path::new(&log_path);
            let mut file = open_log_file(&log_path)
                .map_err(|e| format!("failed to open log file '{}': {e}", log_path))?;
            let mut current_size = file
                .metadata()
                .map_err(|e| format!("failed to inspect log file '{}': {e}", log_path))?
                .len();

            println!(
                "[WRITER] Target: {}, Max Size: {} bytes",
                log_path, max_size
            );

            for received_line in rx_logs {
                let mut line = received_line.into_bytes();
                line.push(b'\n');
                let bytes_to_write = line.len() as u64;

                if current_size.saturating_add(bytes_to_write) > max_size as u64 {
                    rotate(&mut file, log_path_ref, &mut current_size, &tx_archive)?;
                }

                file.write_all(&line)
                    .map_err(|e| format!("failed to write log file '{}': {e}", log_path))?;
                current_size += bytes_to_write;
            }

            if current_size > 0 {
                rotate(&mut file, log_path_ref, &mut current_size, &tx_archive)?;
            }

            println!("[WRITER] Stream ended. Shutting down.");
            Ok(())
        })();

        drop(tx_archive);
        result
    })
}

fn rotate(
    file: &mut File,
    log_path: &Path,
    current_size: &mut u64,
    tx_archive: &Sender<String>,
) -> Result<(), String> {
    file.sync_all()
        .map_err(|e| format!("failed to flush '{}': {e}", log_path.display()))?;

    let rotated_path = next_rotated_path(log_path);
    fs::rename(log_path, &rotated_path).map_err(|e| {
        format!(
            "failed to rotate '{}' to '{}': {e}",
            log_path.display(),
            rotated_path.display()
        )
    })?;

    *file = open_log_file(&log_path.to_string_lossy())
        .map_err(|e| format!("failed to reopen log file '{}': {e}", log_path.display()))?;
    *current_size = 0;

    tx_archive
        .send(rotated_path.to_string_lossy().into_owned())
        .map_err(|_| "archiver thread stopped before rotation completed".to_string())
}
