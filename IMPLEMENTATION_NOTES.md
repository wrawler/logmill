# Logmill Implementation Notes

This document records the reliability improvements made to Logmill, the problems found during review, and the reasoning behind the solutions.

## Summary

The original implementation worked for the basic happy path, but it treated operational failures as programmer errors. File-system errors, archive failures, process restarts, and shutdown signals could therefore cause data loss, silent line drops, or a process that never finished shutting down.

The implementation now prioritizes preserving log data, reporting failures clearly, and making the complete input-to-archive lifecycle testable.

## Problems and Solutions

### 1. Rotated files could be overwritten after a restart

**Problem:**

The writer started `rotation_count` at `1` every time the process launched. If `app.log.1` already existed, the next rotation could replace it. The corresponding archive could also be overwritten.

**Solution:**

The writer now searches for the next unused rotated filename before renaming the active log. Existing files such as `app.log.1` are skipped, so a restart continues with the next available suffix. The archiver also refuses to overwrite an existing archive.

**Reasoning:**

A process-local counter cannot represent files created by earlier processes. Checking the filesystem at the point of rotation makes the naming decision based on the actual persisted state.

### 2. I/O failures could panic worker threads

**Problem:**

Opening files, reading metadata, syncing, renaming, sending archive jobs, running `tar`, and deleting files used `unwrap` or `expect`. A permission error, full disk, missing command, or broken channel could panic a worker thread and terminate the application without a controlled result.

**Solution:**

The writer and archiver now return `Result<(), String>` from their worker threads. The main process joins both workers, reports failures, and exits with a nonzero status.

**Reasoning:**

Logging is an I/O-heavy service. These failures are expected environmental conditions, not impossible programmer states. Returning errors preserves the failure context and lets the caller decide how the process should exit.

### 3. Write failures silently discarded log lines

**Problem:**

A failed `writeln!` was printed to stderr, but processing continued. The failed line was then lost while the program continued to advertise zero data loss.

**Solution:**

The writer uses `write_all` and returns immediately when a write or flush fails. The main process surfaces the worker error through its exit status.

**Reasoning:**

Continuing after an unsuccessful write creates a false sense of success. Failing explicitly is safer than silently accepting incomplete logs.

### 4. Ctrl+C did not perform a real shutdown

**Problem:**

The signal handler only printed a message. The main thread could remain blocked in `stdin.lines()`, and the systemd example used SIGTERM even though the lifecycle did not explicitly coordinate shutdown.

**Solution:**

The signal handler sets an atomic shutdown flag. Stdin is read by a helper thread, while the main thread checks the flag using timed channel receives. Once shutdown begins, the main thread closes the input channel, drains queued log messages through the writer, closes the archive channel, and waits for both workers.

Both SIGINT and SIGTERM are supported through the `ctrlc` handler.

**Reasoning:**

The main thread must be able to observe a signal even when stdin has no data. Separating blocking input from lifecycle coordination allows shutdown to proceed without discarding messages already accepted by the bounded queue.

### 5. The final active log was not archived at EOF

**Problem:**

Only logs rotated before EOF were archived. The final active file remained uncompressed, which made completed runs behave differently depending on whether the size threshold had been reached.

**Solution:**

When the input stream ends or shutdown begins, the writer flushes and archives the remaining non-empty active log, then creates a fresh empty active file for future use.

**Reasoning:**

EOF represents a completed input session. Archiving the final segment makes all completed input follow the same persistence path while retaining a valid active-log path.

### 6. Invalid size values could trigger pathological behavior

**Problem:**

`--max-size 0` was accepted, causing rotation for every line. Multiplication of a numeric value by a unit multiplier could also overflow.

**Solution:**

The parser now rejects zero and uses checked multiplication. Overflow and malformed values produce a command-line parsing error.

**Reasoning:**

Configuration validation should reject values that cannot describe a meaningful rotation policy before worker threads start.

### 7. Archive failures could remove or overwrite the wrong data

**Problem:**

The archiver allowed `tar` to truncate an existing archive path. It also used unchecked filename extraction and removed the source file only after a successful archive command.

**Solution:**

The archiver validates that a rotated path has a filename, refuses an existing archive path, checks the `tar` exit status, and removes the source only after successful archiving.

**Reasoning:**

The source rotated log is the recoverable copy. It must remain available when compression fails, and an existing archive must never be destroyed implicitly.

## Tests Added

### Unit tests

`src/cli.rs` now tests:

- Unit suffixes and whitespace.
- Raw byte values.
- Zero-size rejection.
- Overflow rejection.

### Integration tests

`tests/rotation.rs` now tests:

- Multiple input lines being rotated and archived.
- Creation of an empty active log after final EOF archiving.
- Preservation of an existing `app.log.1` file.
- Preservation of an existing `app.log.1.tar.gz` archive.
- Selection of a later rotation suffix when earlier names already exist.

## Packaging and Documentation Fixes

- `build.sh` now installs `target/release/logmill` instead of looking for a binary in the repository root.
- Installer messages consistently use the `logmill` executable name.
- `PKGBUILD` now matches the Cargo version and has the required `LICENSE` file available.
- The README now uses the correct `cd logmill` path.
- README and systemd documentation describe SIGINT/SIGTERM handling and final-log archiving.

## Verification

The following checks pass from the `logmill` directory:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
bash -n build.sh
```

The test suite currently includes four passing tests: two parser unit tests and two rotation integration tests.

## Remaining Considerations

- Archiving still depends on the system `tar` command being installed and available in `PATH`.
- The current integration tests verify archive creation and preservation of filenames. They do not yet extract every archive and compare every original line byte-for-byte.
- Packaging should still be verified with `makepkg` in an Arch Linux build environment.
- Archive retention is not implemented; this change prevents accidental overwrites but does not delete old archives.
