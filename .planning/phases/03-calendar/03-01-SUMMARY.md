---
phase: 03-01
plan: 01
subsystem: calendar
tags: [database, types, chrono, sqlite]
depends_on:
  requires: [01-02, 02-01]
  provides: [calendar-data-layer, calendar-events-crud]
  affects: [03-02, 03-03]
tech-stack:
  added: [chrono-tz@0.10]
  patterns: [CRUD-impl-on-Database, from_row-parsing]
key-files:
  created:
    - src/calendar/mod.rs
    - src/calendar/types.rs
    - src/db/events.rs
  modified:
    - Cargo.toml
    - src/db/connection.rs
    - src/db/mod.rs
    - src/main.rs
decisions:
  - id: calendar-event-struct
    choice: "CalendarEvent with NaiveDate/NaiveTime fields"
    rationale: "Consistent with chrono types, separate date/time for all-day handling"
  - id: unique-constraint
    choice: "UNIQUE(feed_url, event_uid, dtstart_date)"
    rationale: "Allows same event on different dates for recurring event expansion"
  - id: left-join-feeds
    choice: "LEFT JOIN calendar_feeds for event queries"
    rationale: "Attach feed metadata (name, color) without requiring feed entry first"
metrics:
  duration: ~2 minutes
  completed: 2026-02-08
---

# Phase 03 Plan 01: Calendar Data Layer Summary

**One-liner:** SQLite schema for calendar event caching with CalendarEvent struct and CRUD operations.

## What Was Built

### Calendar Module (src/calendar/)

- **types.rs**: `CalendarEvent` struct with:
  - DB id, feed_url, event_uid for identity
  - dtstart_date, dtstart_time for timing (NaiveTime=None for all-day)
  - dtend_date, dtend_time for duration
  - all_day boolean, rrule for recurrence
  - feed_name, feed_color for display metadata
  - `time_display()` method for formatted time ranges

- **mod.rs**: Exports `CalendarEvent`

### Database Schema

- **calendar_events table**: Caches parsed iCal events
  - UNIQUE(feed_url, event_uid, dtstart_date) allows recurring instances
  - Indexes on dtstart_date and feed_url for queries
  - cached_at timestamp for staleness checking

- **calendar_feeds table**: Feed metadata storage
  - url as PRIMARY KEY
  - name, color from config
  - last_refresh, last_error for sync status

### CRUD Operations (src/db/events.rs)

- `save_calendar_event()`: INSERT OR REPLACE for caching
- `get_calendar_events_for_date()`: Query events with feed metadata via LEFT JOIN
- `clear_feed_events()`: Delete all events for a feed before re-sync
- `save_feed()` / `get_feed()`: Feed metadata CRUD
- `CachedFeed` struct for feed metadata

## Commits

| Hash | Message |
|------|---------|
| ada7ab1 | feat(03-01): add calendar dependencies and create calendar module |
| 58e71d5 | feat(03-01): create calendar database schema and CRUD operations |

## Verification

- [x] `cargo build` compiles without errors
- [x] `cargo run` starts app, database opens (schema created)
- [x] Database has new tables: calendar_events, calendar_feeds
- [x] CalendarEvent struct has all required fields
- [x] CRUD operations available for events

## Deviations from Plan

None - plan executed exactly as written.

## Next Phase Readiness

**Ready for 03-02:** iCal parsing implementation
- CalendarEvent struct ready to receive parsed data
- Database CRUD ready for cache persistence
- Dependencies for parsing (calcard, reqwest) to be added in 03-02

