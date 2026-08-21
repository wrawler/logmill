<div align="center">
  <img src="assets/logo.png" alt="Logmill Logo" width="300"/>
  
  **A lightning-fast, zero-data-loss proxy log rotator built in Rust.**
  
  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT) [![AUR Package](https://img.shields.io/badge/AUR-logmill--git-1793d1.svg)](https://aur.archlinux.org/) [![Fedora COPR](https://img.shields.io/badge/Fedora-COPR-294172.svg)](https://copr.fedorainfracloud.org/)
</div>

Designed for DevOps and QA environments, this utility safely captures standard output (`stdout`) from any application, rotates it based on human-readable file sizes, and automatically compresses old logs in a separate background thread.

## Key Features

*   **Zero Data Loss:** Safely intercepts `SIGINT` (Ctrl+C) to ensure memory buffers are flushed to disk before shutting down.
*   **Memory Safe:** Uses bounded `mpsc` channels to strictly cap RAM usage, preventing Out-Of-Memory (OOM) crashes even during heavy disk I/O.
*   **Non-Blocking Archiving:** Spawns a dedicated background thread to compress old logs (`.tar.gz`) without slowing down your live application's data stream.
*   **Human-Readable CLI:** Easily set size thresholds using formats like `500K`, `10M`, or `2G`.

## Installation

To compile the binary from the source, you will need [Rust and Cargo](https://rustup.rs/) installed. 

We have provided a streamlined build script that compiles the project in optimized release mode and extracts the binary for immediate use:

```bash
git clone [https://github.com/yourusername/log_rotator.git](https://github.com/yourusername/log_rotator.git)
cd log_rotator
chmod +x build.sh
./build.sh
```

## Quick Start / Usage

Because this acts as a proxy utility, you simply pipe (|) the output of your application directly into the rotator.

```bash
# Basic usage (defaults to 1MB rotation on app.log)
node my_server.js | log_rotator

# Advanced usage with custom sizes and an archive directory
python3 my_app.py | log_rotator --file /var/log/my_app.log --max-size 10M --archive-dir /var/log/archives
```

### CLI Options

```
Usage: log_rotator [OPTIONS]

Options:
  -f, --file <FILE>              The path to the active log file [default: app.log]
  -m, --max-size <MAX_SIZE>      Maximum size before rotating (e.g., 10M, 500K) [default: 1M]
  -a, --archive-dir <ARCHIVE_DIR> Directory to save the compressed .tar.gz archives [default: .]
  -h, --help                     Print help
```

## Running as a Background Daemon (systemd)

`LogMill` is designed to be fully compatible with Linux service managers. However, because systemd does not natively interpret the | pipe character, you must wrap your execution pipeline in a bash shell.

To run your application and LogMill continuously in the background, create a service file (e.g., `/etc/systemd/system/myapp.service`) using this template:

```ini
[Unit]
Description=My Application with Rust Proxy Log Rotator
After=network.target

[Service]
Type=simple
User=your_user
Group=your_group

# Wrap the pipeline in a bash session to evaluate the pipe operator
ExecStart=/bin/bash -c '/usr/bin/python3 /path/to/your/app.py | /usr/local/bin/log_rotator --file /path/to/logs/app.log --max-size 50M --archive-dir /path/to/logs/archives'

Restart=on-failure
RestartSec=3

# Ensure graceful shutdown of the Rust daemon
KillSignal=SIGTERM
TimeoutStopSec=10

[Install]
WantedBy=multi-user.target
```

Reload the daemon and start your service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now myapp.service
```

## Contributing

Contributions, issues, and feature requests are welcome!
The project structure is split into modular components (cli.rs, writer.rs, archiver.rs) to make feature development and unit testing straightforward.

## License

This project is licensed under the MIT License - see the LICENSE file for details.