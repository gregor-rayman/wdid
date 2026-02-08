# Phase 3: Calendar Integration - Research

**Researched:** 2026-02-08
**Domain:** iCalendar parsing, async HTTP, egui synchronized scrolling
**Confidence:** HIGH

## Summary

Calendar integration requires parsing iCalendar feeds, fetching them asynchronously without blocking the UI, caching for offline display, and rendering events alongside diary entries in a synchronized two-column layout.

The existing codebase already has infrastructure in place: `CalendarFeed` config type, `event_uid`/`event_snapshot` columns in diary_entries, and a TEA-like architecture. The main additions needed are the iCal parsing stack, async runtime integration, a calendar_events cache table, and the synchronized scroll UI pattern.

**Primary recommendation:** Use calcard for iCal parsing, tokio + reqwest in a background thread with std::sync::mpsc channels to communicate with egui, and track scroll offset in shared state to synchronize two ScrollAreas.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| calcard | 0.3.2 | iCalendar parsing | From Stalwart Labs, liberal parsing (Postel's law), handles VEVENT, VTIMEZONE |
| rrule | 0.14.0 | RRULE expansion | RFC 5545 compliant, chrono integration, handles complex recurrence |
| reqwest | 0.12.x | HTTP client | Async, rustls-tls avoids OpenSSL dependency |
| tokio | 1.x | Async runtime | Industry standard, required by reqwest |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | 0.4.x | Already in use | Date/time handling for events |
| chrono-tz | 0.10.x | Timezone support | Converting VTIMEZONE to system time |

### Already Present
- egui 0.32+ (current: uses 0.33.3 APIs)
- rusqlite 0.34.0 with bundled SQLite
- eframe for desktop integration

**Installation:**
```bash
cargo add calcard@0.3.2 rrule@0.14.0 reqwest@0.12 --features reqwest/rustls-tls tokio@1 --features tokio/rt-multi-thread,tokio/sync
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── calendar/
│   ├── mod.rs           # Re-exports
│   ├── parser.rs        # calcard parsing, RRULE expansion
│   ├── fetcher.rs       # Async HTTP fetch logic
│   └── types.rs         # CalendarEvent struct
├── db/
│   └── events.rs        # calendar_events table operations
└── ui/
    └── timeline.rs      # Modified for two-column layout
```

### Pattern 1: Async + egui Integration
**What:** Run tokio runtime in separate thread, use channels for communication
**When to use:** Any background HTTP fetch that shouldn't block UI
**Example:**
```rust
// In main.rs or app initialization
let rt = tokio::runtime::Runtime::new().unwrap();
let _enter = rt.enter();
std::thread::spawn(move || {
    rt.block_on(async { loop { tokio::time::sleep(Duration::from_secs(3600)).await; } })
});

// In WdidApp struct
struct WdidApp {
    calendar_tx: std::sync::mpsc::Sender<CalendarCommand>,
    calendar_rx: std::sync::mpsc::Receiver<CalendarResult>,
}

// In update(): check for results
if let Ok(result) = self.calendar_rx.try_recv() {
    match result {
        CalendarResult::Events(events) => { /* update cache */ }
        CalendarResult::Error(e) => { /* show error */ }
    }
    ctx.request_repaint();
}
```

### Pattern 2: Synchronized Scroll Areas
**What:** Two ScrollAreas sharing a single scroll offset
**When to use:** Calendar events column + diary entries column
**Example:**
```rust
// In ViewState or similar
struct TimelineState {
    scroll_offset: f32,
}

// In timeline UI
let mut new_offset = state.scroll_offset;

ui.columns(2, |cols| {
    // Left column: calendar events
    let left = ScrollArea::vertical()
        .scroll_offset(Vec2::new(0.0, state.scroll_offset))
        .show(&mut cols[0], |ui| { /* render events */ });
    
    // Right column: diary entries  
    let right = ScrollArea::vertical()
        .scroll_offset(Vec2::new(0.0, state.scroll_offset))
        .show(&mut cols[1], |ui| { /* render entries */ });
    
    // Update from whichever was scrolled
    if left.state.offset.y != state.scroll_offset {
        new_offset = left.state.offset.y;
    } else if right.state.offset.y != state.scroll_offset {
        new_offset = right.state.offset.y;
    }
});

state.scroll_offset = new_offset;
```

### Anti-Patterns to Avoid
- **Blocking UI thread:** Never call `.await` or blocking HTTP in egui's update()
- **Parsing on UI thread:** Parse iCal data in background thread, send parsed events
- **Re-fetching every frame:** Cache events in SQLite, only refresh periodically

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| iCal parsing | Custom parser | calcard | iCal format is complex, edge cases abound |
| RRULE expansion | Manual recurrence | rrule crate | RFC 5545 has many edge cases (BYDAY, BYMONTH, etc.) |
| HTTP client | std::net | reqwest | Async, TLS, redirects, timeouts handled |
| Date math | Manual calculation | chrono | Leap years, DST, timezone conversions |

**Key insight:** iCalendar format appears simple but has many vendor-specific quirks. calcard uses Postel's law (liberal parsing) which handles real-world feeds better than strict parsers.

## Common Pitfalls

### Pitfall 1: Timezone Handling
**What goes wrong:** Events display at wrong times
**Why it happens:** iCal uses VTIMEZONE components, not standard tz database names
**How to avoid:** Parse VTIMEZONE, map to chrono-tz, convert to local time for display
**Warning signs:** Events off by hours, especially around DST transitions

### Pitfall 2: RRULE Infinite Expansion
**What goes wrong:** Memory exhaustion or hangs when expanding recurring events
**Why it happens:** Some events recur forever (e.g., "every Monday")
**How to avoid:** Always limit expansion to a date range (e.g., visible month + buffer)
**Warning signs:** App freezes when loading calendars with recurring events

### Pitfall 3: Stale Cache Display
**What goes wrong:** User sees outdated events after calendar was updated
**Why it happens:** Cache not refreshed, no visual indicator of staleness
**How to avoid:** Store last_refresh timestamp per feed, show "last synced" indicator
**Warning signs:** User complains events are missing or wrong

### Pitfall 4: Scroll Sync Fighting
**What goes wrong:** Scroll areas fight each other, causing jitter
**Why it happens:** Both columns update scroll offset on same frame
**How to avoid:** Detect which column user is hovering, only accept input from that one
**Warning signs:** Scroll feels "sticky" or jumps back

## Code Examples

### Parsing iCal with calcard
```rust
use calcard::ical::ICalendar;

fn parse_ical(data: &str) -> Vec<CalendarEvent> {
    let calendar = ICalendar::parse(data.as_bytes());
    let mut events = Vec::new();

    for component in calendar.components {
        if let Component::Event(vevent) = component {
            let uid = vevent.uid.map(|u| u.to_string());
            let summary = vevent.summary.map(|s| s.to_string());
            let dtstart = vevent.dtstart; // DateOrDateTime
            let dtend = vevent.dtend;
            let rrule = vevent.rrule; // Option for recurring

            events.push(CalendarEvent { uid, summary, dtstart, dtend, rrule });
        }
    }
    events
}
```

### Expanding RRULE
```rust
use rrule::{RRuleSet, Tz};
use chrono::{DateTime, Utc};

fn expand_recurring(rrule_str: &str, start: DateTime<Utc>, range_end: DateTime<Utc>) -> Vec<DateTime<Tz>> {
    let rrule_set: RRuleSet = format!("DTSTART:{}\nRRULE:{}",
        start.format("%Y%m%dT%H%M%SZ"), rrule_str)
        .parse()
        .unwrap();

    rrule_set
        .into_iter()
        .take_while(|dt| *dt <= range_end)
        .take(100) // Safety limit
        .collect()
}
```

### Async Fetch Pattern
```rust
// Command sent to background thread
enum CalendarCommand {
    Refresh(Vec<CalendarFeed>),
}

// Result sent back to UI
enum CalendarResult {
    Events { feed_url: String, events: Vec<CalendarEvent> },
    Error { feed_url: String, error: String },
}

// Background task
async fn fetch_feed(client: &reqwest::Client, feed: &CalendarFeed) -> Result<String, reqwest::Error> {
    client.get(&feed.url)
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await
}
```

## Database Schema Additions

```sql
-- New table for cached calendar events
CREATE TABLE IF NOT EXISTS calendar_events (
    id INTEGER PRIMARY KEY,
    feed_url TEXT NOT NULL,
    event_uid TEXT NOT NULL,
    summary TEXT,
    dtstart TEXT NOT NULL,  -- ISO 8601
    dtend TEXT,
    all_day INTEGER DEFAULT 0,
    rrule TEXT,             -- Original RRULE for re-expansion
    cached_at TEXT NOT NULL,
    UNIQUE(feed_url, event_uid)
);

-- Track feed sync status
CREATE TABLE IF NOT EXISTS calendar_feeds (
    url TEXT PRIMARY KEY,
    name TEXT,
    color TEXT,
    last_refresh TEXT,
    last_error TEXT
);

CREATE INDEX idx_events_date ON calendar_events(dtstart);
CREATE INDEX idx_events_feed ON calendar_events(feed_url);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| ical-rs crate | calcard | 2024 | calcard is more actively maintained, liberal parsing |
| OpenSSL TLS | rustls | 2023+ | No system dependency, easier cross-compile |
| Sync HTTP | Async tokio+reqwest | Standard | Non-blocking UI essential for desktop apps |

## Open Questions

1. **Event conflict detection**
   - What we know: User wants to see events alongside entries
   - What's unclear: Should we highlight time conflicts visually?
   - Recommendation: Start simple, add conflict highlighting as enhancement

2. **All-day event positioning**
   - What we know: CONTEXT.md says "all-day events at top"
   - What's unclear: How to handle multi-day spanning events
   - Recommendation: Show all-day events in header area above scrollable timeline

## Sources

### Primary (HIGH confidence)
- calcard docs.rs + GitHub README - API, parsing behavior
- egui 0.33.3 docs.rs - ScrollArea::scroll_offset() API
- egui-tokio-example GitHub - async integration pattern

### Secondary (MEDIUM confidence)
- rrule crate docs - RFC 5545 expansion
- reqwest docs - rustls-tls feature

### Tertiary (LOW confidence)
- WebSearch for caching patterns - general best practices

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - verified via docs.rs and GitHub
- Architecture (async pattern): HIGH - verified via egui-tokio-example
- Architecture (scroll sync): MEDIUM - egui API verified, pattern inferred
- Pitfalls: MEDIUM - based on iCal domain knowledge and common issues

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (30 days - stable domain)

