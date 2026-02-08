# Technology Stack

**Project:** wdid (What Did I Do)
**Domain:** Native Linux desktop app for daily work tracking
**Researched:** 2026-02-08

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| egui | 0.32.0 | GUI framework | Best accessibility (AccessKit), pure Rust, active development, excellent markdown widget ecosystem. Immediate mode simplifies state management for timeline views. |
| eframe | 0.32.0 | Native windowing | Official egui integration for native apps, handles event loop and platform integration |
| rusqlite | 0.34.0 | SQLite database | Standard Rust SQLite binding, use `bundled` feature to avoid system dependency |
| tokio | 1.x | Async runtime | Required for reqwest async HTTP, background calendar sync |

### Supporting Libraries

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| egui_commonmark | 0.22 | Markdown rendering | Native egui integration, supports CommonMark + GFM (tables, strikethrough, tasklists). Use `better_syntax_highlighting` feature for code blocks. |
| tray-icon | 0.21.x | System tray | From tauri-apps, cross-platform (Linux/GTK, Windows, macOS). Requires libappindicator on Linux. |
| reqwest | 0.12.x | HTTP client | Industry standard, async. Use `rustls-tls` feature to avoid OpenSSL dependency. |
| calcard | latest | iCal parsing | From Stalwart Labs, comprehensive iCalendar/vCard parser with JSCalendar support. Handles VEVENT, VTODO, recurring events. |
| toml | 0.8.x | Config parsing | Standard TOML parser for Rust |
| serde | 1.x | Serialization | Required by most libraries, derive macros for config/data structs |
| directories | 5.x | XDG paths | Cross-platform config/data directory resolution |
| chrono | 0.4.x | Date/time | Standard datetime library, timezone support |
| uuid | 1.x | Unique IDs | For diary entry and event identifiers |

### Development Tools

| Tool | Purpose |
|------|---------|
| cargo-watch | Auto-rebuild on file changes |
| cargo-clippy | Linting |
| cargo-fmt | Formatting |

## Confidence Levels

| Technology | Confidence | Notes |
|------------|------------|-------|
| egui 0.32.0 | HIGH | Verified via boringcactus 2025 GUI survey (July 2025 release) |
| rusqlite 0.34.0 | HIGH | Verified via web search (March 2025 release) |
| egui_commonmark 0.22 | HIGH | Verified via GitHub README |
| tray-icon 0.21.x | HIGH | Verified via GitHub (Jan 2026 release) |
| reqwest 0.12.x | HIGH | Verified via web search (0.13 available Dec 2025) |
| calcard | MEDIUM | Found via Stalwart Labs, comprehensive but less widely used than icalendar crate |
| tokio 1.x | HIGH | Industry standard async runtime |

## Installation

```bash
# Core dependencies (Cargo.toml)
cargo add egui eframe
cargo add rusqlite --features bundled
cargo add tokio --features rt-multi-thread,macros
cargo add reqwest --features rustls-tls
cargo add egui_commonmark --features better_syntax_highlighting
cargo add tray-icon
cargo add toml serde --features serde/derive
cargo add directories chrono uuid --features uuid/v4

# Linux system dependencies (for tray-icon)
# Debian/Ubuntu:
sudo apt install libgtk-3-dev libappindicator3-dev
# Arch:
pacman -S gtk3 libappindicator-gtk3
```

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| GUI | egui | iced 0.14 | No accessibility support, no IME support. Elm architecture adds complexity for this use case. |
| GUI | egui | Slint | DSL adds build complexity, commercial license for closed source |
| GUI | egui | Dioxus | WebView approach (diet Electron) - unnecessary overhead for native app |
| GUI | egui | Tauri | Full IPC boundary between Rust/JS adds complexity, overkill for this use case |
| GUI | egui | GTK4-rs | No accessibility on Windows (macOS as nice-to-have), significant learning curve |
| iCal | calcard | icalendar | calcard more comprehensive, handles edge cases better, from established org (Stalwart) |
| HTTP | reqwest | ureq | reqwest more widely used, better async support for background sync |
| TLS | rustls | native-tls | Avoid OpenSSL system dependency, pure Rust |

## What NOT to Use

| Technology | Why Avoid |
|------------|-----------|
| GTK4-rs | No accessibility on Windows, complex async story |
| Tauri | Unnecessary IPC boundary for pure Rust app |
| native-tls | OpenSSL dependency complicates builds |
| libsqlite3-sys without bundled | Requires system SQLite, version mismatches |
| Electron | Massive overhead for simple timeline app |

## Version Compatibility

egui ecosystem versions should be aligned:
- egui 0.32.x
- eframe 0.32.x  
- egui_commonmark 0.22 (compatible with egui 0.32)

## Architecture Notes

**Event Loop Integration:**
- egui/eframe provides the main event loop
- tray-icon requires GTK event loop on Linux (eframe uses winit which can coexist)
- Use `TrayIconEvent::set_event_handler` to forward tray events to egui

**Async Pattern:**
- tokio runtime for background calendar sync
- Use channels (crossbeam or tokio::sync::mpsc) to communicate between async tasks and UI thread

## Sources

- boringcactus 2025 Rust GUI survey: https://boringcactus.com/2025/07/27/rust-gui-survey-2025.html
- egui_commonmark GitHub: https://github.com/lampsitter/egui_commonmark
- tray-icon GitHub: https://github.com/tauri-apps/tray-icon
- Stalwart calcard: https://github.com/stalwartlabs/calcard
- reqwest releases: crates.io/crates/reqwest
- rusqlite releases: crates.io/crates/rusqlite

