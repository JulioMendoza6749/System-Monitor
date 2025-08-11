# System-Monitor

## Description
System Monitor is a performance monitoring tool developed in Rust using Cargo.
It collects advanced system metrics on Windows, including:
- CPU: Usage per core, frequency, and temperature (via OpenHardwareMonitor).
- Memory: Physical usage, swap, and cache.
- Network: Traffic per interface (MB/s) and active connections.
- Disk: Read/write (IOPS) and response time.
- Processes: Top 5 processes by resource usage.
- Historical Data Storage: Saves metrics every 5–10 minutes for a configurable retention period.
- Startup Execution: Can be configured to run at system startup.
- Visualization: Generates Python-based graphs for each metric.

The application uses OpenHardwareMonitor to retrieve CPU temperature, as Rust alone cannot access this data directly on Windows.
OpenHardwareMonitor must be running and configured to expose data at: `http://localhost:8085`

## Technologies and Dependencies

Rust Crates
- anyhow — error handling.
- serde — serialize/deserialize structs to/from JSON.
- serde_json — JSON manipulation.
- sysinfo — system information (CPU, memory, etc.).
- netstat — network connections info.
- wmi — system information via WMI on Windows.
- chrono — date/time handling.

Python Libraries
- json (built-in) — JSON parsing.
- matplotlib — graph plotting.
- collections.Counter (built-in) — counting and aggregation.
- re (built-in) — regular expressions.
- os (built-in) — file and system operations.

``

## Installation & Setup
1. Clone the repository
 ```bash
git clone https://github.com/JulioMendoza6749/System-Monitor.git
```

2. Configure OpenHardwareMonitor
Place `OpenHardwareMonitor` in the same project directory or anywhere accessible.
Run it and enable the web server at `http://localhost:8085`.

3. Update variables in main.rs:
 ```rust
// VARIABLES TO CHANGE
const user_cpu: &str ="Intel Core i3-10110U"; // Your CPU name
const var_stop: &str = "2025-04-12 22:07:15"; // Stop date/time for metric collection
const route: &str = "C:\\path\\to\\metrics_data.json"; // Absolute path to JSON file
const route_python: &str = "C:\\path\\to\\gen_metrics.py"; // Absolute path to Python script
 ```
4. Install Python dependencies:
 ```bash
pip install matplotlib pandas
 ```

5. Build and run the Rust application:
 ```bash
cargo run
 ```

## Usage
The Rust program collects metrics and saves them in JSON format every 5–10 minutes until the stop date is reached.
Once the stop date is reached, metric collection stops and the Python script runs automatically to generate graphs from the collected data.

## Notes
- Designed for Windows only.
- Requires OpenHardwareMonitor running with the HTTP server enabled.
- Paths to files and scripts must be absolute in `main.rs`.

## License
This project is licensed under the MIT License — see the LICENSE file for details.






