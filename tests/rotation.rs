use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_directory() -> PathBuf {
    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("logmill-test-{}-{unique_id}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    directory
}

fn run_logmill(directory: &Path, input: &str) {
    let log_path = directory.join("app.log");
    let archive_dir = directory.join("archives");
    let mut child = Command::new(env!("CARGO_BIN_EXE_logmill"))
        .args([
            "--file",
            log_path.to_str().unwrap(),
            "--max-size",
            "8B",
            "--archive-dir",
            archive_dir.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
}

#[test]
fn archives_all_input_lines() {
    let directory = temporary_directory();
    run_logmill(&directory, "first\nsecond\nthird\n");

    let archive_dir = directory.join("archives");
    let archives = fs::read_dir(&archive_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(archives.len(), 3);
    assert_eq!(fs::metadata(directory.join("app.log")).unwrap().len(), 0);
    assert!(!directory.join("app.log.1").exists());

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn does_not_overwrite_existing_rotation() {
    let directory = temporary_directory();
    fs::write(directory.join("app.log"), "old\n").unwrap();
    fs::write(directory.join("app.log.1"), "previous\n").unwrap();
    fs::create_dir(directory.join("archives")).unwrap();
    fs::write(
        directory.join("archives/app.log.1.tar.gz"),
        "existing archive",
    )
    .unwrap();

    run_logmill(&directory, "new\n");

    assert_eq!(
        fs::read_to_string(directory.join("app.log.1")).unwrap(),
        "previous\n"
    );
    assert!(directory.join("archives/app.log.2.tar.gz").exists());
    assert_eq!(
        fs::read_to_string(directory.join("archives/app.log.1.tar.gz")).unwrap(),
        "existing archive"
    );

    fs::remove_dir_all(directory).unwrap();
}
