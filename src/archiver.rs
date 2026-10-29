use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

/// Spawns a background thread to compress rotated log files.
pub fn spawn(rx_archive: Receiver<String>, archive_dir: String) -> JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        (|| -> Result<(), String> {
            fs::create_dir_all(&archive_dir).map_err(|e| {
                format!("failed to create archive directory '{}': {e}", archive_dir)
            })?;

            println!("[ARCHIVER] Ready. Saving archives to: {}", archive_dir);

            for file_to_archive in rx_archive {
                let source_path = Path::new(&file_to_archive);
                let file_name = source_path
                    .file_name()
                    .ok_or_else(|| {
                        format!(
                            "invalid rotated log path '{}': no filename",
                            file_to_archive
                        )
                    })?
                    .to_string_lossy();
                let archive_path = Path::new(&archive_dir).join(format!("{}.tar.gz", file_name));
                let archive_name = archive_path.to_string_lossy();

                if archive_path.exists() {
                    return Err(format!(
                        "refusing to overwrite existing archive '{}'",
                        archive_path.display()
                    ));
                }

                println!(
                    "[ARCHIVER] Compressing {} -> {}",
                    file_to_archive, archive_name
                );

                let status = Command::new("tar")
                    .args(["-czf", archive_name.as_ref(), &file_to_archive])
                    .status()
                    .map_err(|e| format!("failed to execute tar for '{}': {e}", file_to_archive))?;

                if !status.success() {
                    return Err(format!("tar failed while archiving '{}'", file_to_archive));
                }
                fs::remove_file(&file_to_archive).map_err(|e| {
                    format!("failed to remove archived log '{}': {e}", file_to_archive)
                })?;
            }

            println!("[ARCHIVER] Shutting down.");
            Ok(())
        })()
    })
}
