# wdid — What Did I Do

## What This Is

A native Linux desktop application for tracking daily work. It displays imported calendar events alongside personal diary entries in a unified timeline view, allowing users to see their schedule and capture notes, thoughts, and time spent — all in one place.

## Core Value

See your day at a glance and capture what you did — calendar events and personal notes unified in a single timeline that makes it easy to track and recall your work.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Display single-day timeline view with calendar events (left) and diary entries (right)
- [ ] Import and display events from iCal feeds (read-only, up to 3-5 feeds)
- [ ] Create standalone diary entries with start time and optional duration
- [ ] Create diary entries linked to specific calendar events
- [ ] Edit diary entries in place (raw markdown input, rendered display)
- [ ] Support basic markdown formatting in diary entries
- [ ] Support hashtags in entries for categorization
- [ ] Search entries by hashtag across all dates
- [ ] Navigate between days with arrow buttons
- [ ] Manual and automatic (hourly) calendar refresh
- [ ] Store diary entries in SQLite database
- [ ] Configure calendar feeds via TOML config file (~/.config/wdid/config.toml)
- [ ] System tray icon with left-click show/hide, right-click menu
- [ ] Close window minimizes to tray, quit via tray menu
- [ ] Remember window size and position between sessions
- [ ] Delete entries via right-click context menu or by clearing text
- [ ] Small "add note" button on calendar events to create linked entries

### Out of Scope

- Write-back to external calendars — iCal feeds are read-only by design
- Formal billing/invoicing features — time tracking is for personal reference only
- Rich text editor — markdown is sufficient, no WYSIWYG
- Mobile or web versions — desktop Linux is the focus
- Calendar event creation — this is a viewer/diary, not a calendar manager
- Sync between devices — local-only storage

## Context

- Target user: Someone who wants to track what they did each day, with context from their calendar
- Use case: End of day reflection, time tracking for personal awareness, meeting notes
- The name "wdid" stands for "What Did I Do" — the question you answer when reviewing your day
- Hashtags enable finding related entries (e.g., #project-x, #meeting, #billing)

## Constraints

- **Language**: Rust — non-negotiable, core requirement
- **Platform**: Linux primary — Windows/macOS are nice-to-have, not blocking
- **UI Framework**: To be decided — egui, iced, tauri, or slint are candidates
- **Storage**: SQLite — chosen for reliability and queryability
- **Config**: TOML at XDG location — follows Linux conventions

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Two-column timeline layout | Events and notes side-by-side, sorted by time | — Pending |
| SQLite for storage | Single file, queryable, reliable | — Pending |
| TOML config at ~/.config/wdid/ | XDG standard, human-readable | — Pending |
| Markdown for diary entries | Simple, portable, no complex editor needed | — Pending |
| Read-only calendar sync | Simpler implementation, iCal feeds are inherently read-only | — Pending |
| UI framework | To be researched — egui, iced, tauri, slint | — Pending |

---
*Last updated: 2026-02-08 after initialization*

