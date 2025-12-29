# RDAIO v0.1.0 Release

A retro DOS-style MP3 player written in Rust with instant startup and terminal UI.

**Full Changelog:** https://github.com/kpstsp/rdaio-k/commits/v0.1.0

## Core Features

### 🎵 Audio Playback
- Streaming MP3 decoder with instant startup (~100-200ms)
- Play, pause, resume, and stop controls
- Automatic next track when current finishes

### 📁 File Management
- DOS-style folder browser for navigating directories
- Load single files or entire directories of MP3s
- Queue persistence - saves your playlist between sessions
- Clear queue to start fresh

### 🏷️ Metadata & Display
- Reads ID3 tags to display song titles
- Toggle display mode: show titles or filenames
- Real-time track selection with Up/Down arrows

### ⚙️ Configuration
- `--debug` flag for detailed logging
- Optional license documentation (MIT + compatible third-party)

## Keyboard Controls

| Key | Action |
|-----|--------|
| `[P]` | Play |
| `[Z]` | Pause/Resume |
| `[S]` | Stop |
| `[M]` | Toggle display mode (title vs filename) |
| `[F]` | Open folder browser |
| `[C]` | Clear queue |
| `[Q]` | Quit |
| `[Up/Down]` | Select track |

## Technical Highlights

- Written in Rust with **ratatui** TUI framework
- Streaming decoder **(symphonia)** instead of full buffering = fast startup
- Works on **Windows and Linux**
- Minimal dependencies, production-ready code

## License & Source Code

This release includes **Symphonia** (MPL-2.0).

**Source code** for this release (including MPL-2.0 components):
- https://github.com/kpstsp/rdaio-k/archive/refs/tags/v0.1.0.zip
- Or download via GitHub "Source code (zip)" for this tag

### References
- [Symphonia - crates.io](https://crates.io/crates/symphonia-bundle-mp3/0.5.5)
- [Symphonia - GitHub](https://github.com/pdeljanov/Symphonia)
