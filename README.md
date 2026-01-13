# Hecto

A lightweight terminal-based text editor written in Rust.

Learning from [this](https://philippflenker.com/hecto/).

## Features

- **Text Editing**: Full-featured text editing with support for Unicode characters and grapheme clusters
- **File Operations**: Open, edit, and save files with unsaved changes protection
- **Search Functionality**: Search through text with highlighting and navigate between matches
- **Status Bar**: Real-time display of file status, cursor position, and modification state
- **Syntax Highlighting**: Clean, intuitive interface with proper terminal rendering
- **Cross-platform**: Works on Linux, macOS, and Windows

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Build from Source

```bash
git clone <repository-url>
cd hecto
cargo build --release
```

The compiled binary will be located at `target/release/hecto`.

## Usage

### Starting the Editor

```bash
# Open a new empty buffer
./hecto

# Open an existing file
./hecto filename.txt
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl-F` | Find/Search in file |
| `Ctrl-N` | Find next match |
| `Ctrl-S` | Save file |
| `Ctrl-T` | Quit editor |
| `Esc` | Dismiss/Cancel current prompt |
| Arrow Keys | Navigate through text |
| `Home/End` | Move to start/end of line |
| `Page Up/Down` | Scroll up/down by page |

### Saving Files

- If editing an existing file, press `Ctrl-S` to save
- For new files, `Ctrl-S` will prompt for a filename
- Press `Esc` to cancel save operation

### Searching

1. Press `Ctrl-F` to open search prompt
2. Type your search query
3. Press `Enter` to confirm
4. Use `Ctrl-N` to jump to next match
5. Press `Esc` to exit search mode

### Quitting

- Press `Ctrl-T` to quit
- If file has unsaved changes, you'll need to press `Ctrl-T` three times to confirm quit

## Dependencies

- **crossterm** (0.29.0): Cross-platform terminal manipulation library
- **unicode-segmentation** (1.12.0): Unicode text segmentation
- **unicode-width** (0.2.2): Display width of Unicode characters

## Architecture

The editor is organized into modular components:

- **Editor**: Main application controller
- **View**: Document viewing and editing logic
- **Terminal**: Low-level terminal operations
- **UI Components**: Status bar, message bar, command bar
- **Document**: Text buffer management
- **Line**: Individual line handling with Unicode support

## Development

### Running in Debug Mode

```bash
cargo run
cargo run -- filename.txt
```

### Code Quality

The project uses strict Clippy lints for code quality:

```bash
cargo clippy
```

### Building for Release

```bash
cargo build --release
```

## License

This project is part of the learning exercise.

## Contributing

This is a personal learning project, but suggestions and improvements are welcome!
