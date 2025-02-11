# Diary Reminder App

## Overview
**Diary Reminder** is an open-source diary application built in pure Rust, designed to help users offload distracting thoughts and maintain focus on productive work. Instead of just reminding users, the app opens Vim (on Linux) or Notepad (on Windows) on top of the screen, allowing users to enter whatever they want, review their to-do list, update it, or capture stray thoughts.

## Features
- 🔔 **Automatic Editor Launch**: Opens Vim (Linux) or Notepad (Windows) for quick note-taking.
- 📝 **Distraction Parking**: Offload triggering thoughts for later reflection.
- 🔒 **Privacy First**: Your diary stays local—no cloud sync, no tracking.
- ☁️ **Backup Support**: Optionally backup your diary to Google Drive or Notion.
- ⚡ **Lightweight & Fast**: Built with Rust for efficiency and minimal resource usage.
- 🌍 **Open Source**: Community-driven development.

## Installation
### Prerequisites
Ensure you have Rust installed. You can install it via:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone and Build
```sh
git clone https://github.com/yourusername/diary-reminder.git
cd diary-reminder
cargo build --release
```

### Running the App
```sh
target/release/diary-reminder
```

## Usage
1. **Set Reminder Frequency**: Choose how often you want to be reminded.
2. **Automatic Note-Taking**: When notified, Vim (Linux) or Notepad (Windows) opens automatically.
3. **Enter and Review**: Jot down thoughts, review the to-do list, update tasks, or capture new ideas.
4. **Backup Your Diary**: Optionally sync your entries with Google Drive or Notion for safekeeping.

## Contributing
We welcome contributions! Feel free to submit issues, feature requests, or pull requests.

### Development Setup
```sh
git clone https://github.com/yourusername/diary-reminder.git
cd diary-reminder
cargo run
```

## License
This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments
A huge thanks to the Rust community for their continued support and contributions.

