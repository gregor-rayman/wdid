# wdid — What Did I Do

## What This Is

A native Linux desktop application for tracking daily work. It displays imported calendar events alongside personal diary entries in a unified timeline view, allowing users to see their schedule and capture notes, thoughts, and time spent — all in one place.

## Core Value

See your day at a glance and capture what you did — calendar events and personal notes unified in a single timeline that makes it easy to track and recall your work.

## Current State

**Version:** v1.0 (shipped 2026-02-08)

### What's Working

- ✓ Two-column timeline: calendar events (left), diary entries (right)
- ✓ iCal feed import with RRULE expansion (up to 5 feeds)
- ✓ Create/edit/delete diary entries with markdown rendering
- ✓ Link diary entries to calendar events
- ✓ Hashtag and full-text search across all dates
- ✓ Day navigation with arrow buttons
- ✓ Manual and hourly automatic calendar refresh
- ✓ SQLite database with WAL mode
- ✓ TOML config at ~/.config/wdid/config.toml
- ✓ System tray with show/hide toggle
- ✓ Close-to-tray behavior
- ✓ Window size/position persistence
- ✓ Export to Markdown/JSON
- ✓ Standup and weekly retro summaries

### Tech Stack

| Component | Choice |
|-----------|--------|
| Language | Rust |
| GUI | egui/eframe |
| Database | SQLite (rusqlite) |
| Calendar | ical + rrule crates |
| HTTP | reqwest + tokio |
| System Tray | ksni |
| Clipboard | arboard |
| File Dialogs | rfd |

## Next Milestone Goals

(To be defined with `/gsd:new-milestone`)

## Out of Scope

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

---
*Last updated: 2026-02-08 after v1.0 milestone completion*

