# lw - CLI Personal Log Manager

`lw` is a terminal-based tool for creating and managing personal logs directly from the command line. It enables you to quickly record, edit, and review your achievements, notes, or any personal entries using an interactive text user interface (TUI).

---

## Features

- Create and maintain a personal log of accomplishments, notes, or reflections  
- Interactive terminal UI with keyboard-driven navigation and editing  
- Add, edit, and remove entries via popup editor  
- Entries are timestamped and sorted by creation date (most recent first)  
- Persistent storage of logs in a JSON file under a platform-specific config directory  
- Lightweight and easy to use from any terminal session  

---

## Installation

Download a binary from the Releases page or build from source.

### Prerequisites

- Rust toolchain (1.88+ recommended)  
- Cargo package manager  

### Build from source

Clone the repository and build:

`git clone https://git.itsscb.de/itsscb/lw.git`  
`cd lw`  
`cargo build --release`

The compiled binary will be located at `target/release/lw`.

---

## Usage

Run the application from the terminal:

`./target/release/lw`

## Data Storage

Your personal log entries are saved in a JSON file named `config.json` located in the platform-specific configuration directory:

- **Windows:** `%APPDATA%\lw\config.json`  
- **Unix/Linux/macOS:** `$HOME/.config/lw/config.json`  

The directory and file are created automatically on first run.

---

## Contributing

Contributions, bug reports, and feature requests are welcome. Please open issues or submit pull requests.

---

For support or questions, please refer to the project repository.
