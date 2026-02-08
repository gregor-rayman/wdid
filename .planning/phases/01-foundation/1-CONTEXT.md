# Phase 1: Foundation & Data Layer - Context

**Gathered:** 2026-02-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish reliable data persistence and configuration infrastructure that all features depend on. Delivers:
- SQLite database for diary entries (DIARY-05, SYS-02)
- Config file at XDG location ~/.config/wdid/config.toml (SYS-07)
- App creates directories/files automatically on first run
- Invalid config shows helpful error (doesn't crash)

</domain>

<decisions>
## Implementation Decisions

### GUI Framework
- Use egui (with eframe) — locked decision
- Version 0.32.0 as recommended by research

### Database Schema
- Fields: id, date, start_time, duration (optional), content, created_at, updated_at
- Event linking: store both event_uid (for matching) AND event_snapshot (title + time for display resilience)
- Tags: extracted from content on-the-fly, not stored separately
- Ordering: same start_time → use created_at (creation order)
- Deletion: hard delete, no history/soft delete

### Config File Structure
- Minimal config — only calendar feeds
- Per-feed fields: url (required), name, color
- Use calendar metadata as defaults for name/color
- User modifies by editing config file directly
- First-run: create config with commented-out examples

```toml
# Add calendar feeds below:
# [[calendars]]
# url = "https://calendar.google.com/calendar/ical/..."
# name = "Work"
# color = "#3b82f6"
```

### Config Error Handling
- Invalid/unreachable calendar URL: show warning, continue loading others
- Config parse error: show friendly message (no raw TOML errors)

### First-Run Behavior
- Silent directory/file creation
- Welcome message in empty timeline area (where entries would appear)
- Warm tone: "Welcome to wdid! Start by adding a diary entry, or configure calendar feeds in ~/.config/wdid/config.toml"
- One-time only — never shows again after first launch

### Error Handling
- Errors shown in-app only (no automatic log files)
- Friendly messages, not raw errors
- Database corruption: offer recovery dialog ("Start fresh? Old file will be backed up")
- Optional logging: enable via --verbose flag

### Claude's Discretion
- Exact SQLite schema SQL (column types, indexes)
- Database file location within XDG data dir
- Log file location and rotation (when --verbose enabled)
- Exact welcome message wording

</decisions>

<specifics>
## Specific Ideas

- Config should feel like a simple dotfile — minimal, not overwhelming
- Recovery dialog for corrupted DB should be reassuring, not scary
- Welcome message should guide without being intrusive

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-02-08*

