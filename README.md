# telemetry-daemon

A Linux telemetry stack with three main parts:

- telemetry-worker: collects process and system metrics and stores them in SQLite
- daemon: launches the worker in the background
- data-analyzer: reads the stored telemetry and generates reports and visualizations

The project is centered on the data-analyzer and the telemetry pipeline it consumes.

## Core components

### telemetry-worker

The Rust worker scans the proc filesystem, samples CPU and memory usage, and writes telemetry to a local SQLite database.

### daemon

The C launcher detaches the worker into the background, redirects logs, and keeps the telemetry process running as a daemon.

### data-analyzer

The Python analyzer is the main insight layer. It turns raw telemetry into charts, anomaly warnings, process summaries, and stability reports.

The worker reads from Linux system files such as /proc/stat, /proc/meminfo, /proc/<pid>/status, and /etc/passwd, so it is intended for Linux only.

## What it collects

- Per-process PID
- Process owner username
- Process command name
- Per-process CPU usage
- Per-process memory usage
- System-wide CPU usage
- System-wide memory usage

## How it works

- The worker scans all numeric entries in /proc to find running processes.
- It reads process metadata and computes CPU usage using tick deltas over a 1 second sampling window.
- It records telemetry every 15 seconds.
- It performs cleanup every 30 minutes and removes data older than 24 hours.
- Data is stored in a SQLite database at /var/lib/teld/telemetry.db.

## Requirements

- Linux
- Rust toolchain with Cargo
- A C compiler such as clang
- Permission to create and write to /var/lib/teld
- Permission to install the worker in /usr/local/bin

## Build

Use the provided Makefile:

- Build both binaries: make
- Build only the Rust worker: cargo build --release
- Build only the launcher: make launcher
- Build and run the visualizer: make visualizer

The release worker binary is built as target/release/teld-worker.

## Install

The launcher expects the worker to be available at /usr/local/bin/teld-worker.

Recommended install flow:

1. Build the project with make or cargo build --release.
2. Copy the worker into place:
	- sudo cp target/release/teld-worker /usr/local/bin/teld-worker

The Makefile also provides an install target that performs the copy.

## Run

The launcher starts the worker as a background process and redirects logs to /tmp/teld-worker.log.

To run with the Makefile:

- make run

This will:

- build the launcher
- build the Rust worker
- create /var/lib/teld if needed
- install the worker to /usr/local/bin/teld-worker
- launch the daemon

To inspect the collected data visually:

- make visualizer

## Data storage

The worker creates the SQLite database automatically and initializes these tables:

- telemetry
- system_metrics

It also creates indexes on timestamp, pid, and command for faster queries.

## Logs

- Worker log file: /tmp/teld-worker.log
- Database file: /var/lib/teld/telemetry.db

## Cleanup and uninstall

Makefile targets:

- make clean — remove build artifacts and the worker log
- make clean-data — remove /var/lib/teld
- make uninstall — remove the installed worker and data directory

## Data Analyzer

The data-analyzer is the most important user-facing tool in this repository. It reads the telemetry database and turns raw records into actionable insight.

### Features

- System metrics dashboard: CPU and memory trends over time
- System metrics visualization: plots CPU and memory usage over time
- Per-process analysis: bar charts of average CPU and memory usage by command
- Anomaly detection: logs warnings when CPU or memory usage exceeds thresholds
- Process heatmap: visualizes top 10 processes' CPU usage patterns over time
- CPU vs memory scatter plot: shows resource correlation for all processes
- Process stability analysis: reports standard deviation of CPU and memory usage
- Recurring processes report: identifies processes that appear frequently in the data

### Main output

The analyzer produces:

- visual dashboards for process and system behavior
- anomaly warnings for unusual resource usage
- recurring process summaries
- stability metrics for long-term analysis

### Requirements

- Python 3.x
- pandas
- matplotlib
- numpy

Install dependencies:

```
pip install -r requirements.txt
```

### Run

To analyze collected telemetry data:

```
python -m data-analyzer
```

The analyzer reads from the SQLite database at /var/lib/teld/telemetry.db and outputs:

- Visualization plots to the screen
- Detailed logs to data-analyzer/out/analyzer.log
- Anomaly warnings and process reports to the log file

## Project layout

- src/main.rs — telemetry-worker entry point
- src/collector.rs — process and system sampling logic for telemetry-worker
- src/logger.rs — SQLite storage and cleanup logic for telemetry-worker
- src/process.rs — process model
- src/system.rs — system model and /proc helpers
- daemon/launcher.c — daemon launcher
- data-analyzer/__main__.py — data-analyzer main entry point and visualization tool

## Notes

- The project uses rusqlite with the bundled SQLite feature, so no system SQLite development package is required.
- The launcher currently expects the worker at /usr/local/bin/teld-worker. If you install it elsewhere, update the launcher or your install path accordingly.
- CPU and memory percentages are approximate and based on sampled proc data.
