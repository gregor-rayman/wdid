---
phase: 03-calendar
plan: 03
subsystem: calendar
tags: [tokio, reqwest, async, http, channels]
status: complete
dependency-graph:
  requires: [03-01]
  provides: [async-calendar-fetch, calendar-worker-channels]
  affects: [03-04, 03-05]
tech-stack:
  added: [tokio@1, reqwest@0.12]
  patterns: [background-worker-thread, channel-communication, non-blocking-ui]
key-files:
  created:
    - src/calendar/fetcher.rs
  modified:
    - Cargo.toml
    - src/calendar/mod.rs
    - src/app.rs
decisions:
  - decision: rustls-tls for reqwest
    rationale: Avoids OpenSSL system dependency
metrics:
  duration: ~3 minutes
  completed: 2026-02-08
---

# Phase 03 Plan 03: Async Calendar Fetching Summary

**One-liner:** Tokio-based background worker fetches iCal feeds asynchronously via channels without blocking UI

## What Was Built

### Calendar Fetcher Module (`src/calendar/fetcher.rs`)

- **CalendarCommand enum:** Commands to send to worker (RefreshAll, RefreshOne, Shutdown)
- **CalendarResult enum:** Results from worker (FeedData, FeedError, RefreshComplete)
- **spawn_calendar_worker():** Spawns background thread with tokio multi-thread runtime
- **worker_loop():** Async loop processing commands and fetching feeds
- **fetch_feed():** Async HTTP GET with 30s timeout using reqwest

### WdidApp Integration (`src/app.rs`)

- Added `calendar_tx`/`calendar_rx` channel fields
- Spawns worker on app startup
- Non-blocking `try_recv()` polling in update loop
- Triggers RefreshAll for configured feeds on first update
- Logs results to stderr (parsing/caching in Plan 04)

## Technical Decisions

| Decision | Rationale |
|----------|-----------|
| rustls-tls feature | Avoids OpenSSL system dependency, better portability |
| Dedicated thread for runtime | Keeps tokio runtime isolated from UI thread |
| std::sync::mpsc channels | Simple, works well with blocking recv in worker |
| 30s timeout | Reasonable for network requests, prevents hangs |
| 2 worker threads | Enough for parallel feed fetches |

## Commits

| Hash | Message |
|------|---------|
| 7f75a54 | feat(03-03): add async dependencies |
| 30c6c74 | feat(03-03): create calendar fetcher with background worker |
| 0649e09 | feat(03-03): wire fetcher channels into WdidApp |

## Verification

- ✅ `cargo build` compiles without errors
- ✅ App with calendar URL in config fetches data (11675 bytes received)
- ✅ App doesn't freeze during network request
- ✅ Results logged: "Received X bytes from URL"
- ✅ RefreshComplete message received after batch

## Deviations from Plan

None - plan executed exactly as written.

## Integration Points

### For Plan 03-04 (iCal Parsing)
- CalendarResult::FeedData contains raw iCal text in `data` field
- Parse using calcard crate, store via database CRUD from 03-01

### For Plan 03-05 (Timeline Integration)
- Poll results in update() already triggers ctx.request_repaint()
- After parsing/caching, reload calendar events for current date

## Next Phase Readiness

✅ Ready for 03-04 (iCal parsing):
- Async fetch infrastructure complete
- Channel communication working
- FeedData results contain raw iCal for parsing

