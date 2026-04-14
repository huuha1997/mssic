# MSSIC

> High-performance terminal music player for macOS with native menu bar integration.

![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust)
![Ratatui](https://img.shields.io/badge/TUI-Ratatui-blue)
![macOS](https://img.shields.io/badge/Platform-macOS-lightgrey?logo=apple)

## Features

- **Multi-tab Interface** — Search, Library, Queue with keyboard navigation
- **Real-time Search** — Fast music search via yt-dlp integration
- **10-band Equalizer** — Visual EQ adjustment with panel toggle
- **Persistent Library** — Track metadata saved to disk
- **Auto-queue** — Automatic progression to next track
- **Smart Rendering** — 250ms refresh when playing, 1s when idle
- **macOS Native** — Menu bar app with proper `.app` bundle
- **Keyboard-driven** — Full control without mouse

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (2024 edition) |
| TUI | Ratatui + Crossterm |
| Async | Tokio |
| Audio Source | yt-dlp |
| Menu Bar | Swift (AppKit) |
| Build | Makefile → .app bundle |

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `i` | Search |
| `e` | Toggle Equalizer |
| `Tab` | Switch tabs |
| `Enter` | Play / Select |
| `Space` | Pause / Resume |
| `n` | Next track |
| `q` | Quit |
| `?` | Help |

## Project Structure

```
src/
├── main.rs        # Event loop (50ms input, 500ms IPC)
├── app.rs         # Application state
├── player.rs      # Audio playback
├── equalizer.rs   # 10-band EQ
├── library.rs     # Persistent storage
├── queue.rs       # Playback queue
├── ytdlp.rs       # yt-dlp integration
└── ui/            # Terminal rendering
menubar/           # Swift menu bar app
build/             # macOS .app output
```

## Getting Started

```bash
# Prerequisites: Rust, yt-dlp
brew install yt-dlp

# Build and run
make build
make run

# Or build .app bundle
make app
```

## License

MIT
