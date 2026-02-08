---
phase: 03-calendar
plan: 02
subsystem: calendar-parsing
tags: [ical, rrule, calcard, parsing, recurrence]
dependency-graph:
  requires: ["03-01"]
  provides: ["parse_ical function", "RRULE expansion"]
  affects: ["03-04", "03-05"]
tech-stack:
  added: ["calcard@0.3", "rrule@0.14"]
  patterns: ["liberal iCal parsing", "date range filtering"]
key-files:
  created:
    - src/calendar/parser.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/calendar/mod.rs
decisions:
  - id: calcard-icalendar-api
    choice: "Use calcard's ICalendar module with VEvent component type"
    rationale: "calcard 0.3.2 uses ICalendarComponentType::VEvent (not Event)"
  - id: rrule-date-filtering
    choice: "Use RRuleSet with after/before filters"
    rationale: "Efficient date range expansion without generating all occurrences"
  - id: safety-limit
    choice: "Cap RRULE expansion at 100 occurrences"
    rationale: "Prevents infinite expansion from malformed/unbounded RRULEs"
metrics:
  duration: "~15 minutes"
  completed: "2026-02-08"
---

# Phase 03 Plan 02: iCal Parsing Summary

**One-liner:** iCal parsing with calcard and RRULE expansion using rrule crate, producing CalendarEvent structs from raw feed data

## Objective

Convert raw iCal data into CalendarEvent structs ready for caching and display, with proper handling of single events, all-day events, and recurring events.

## Tasks Completed

| # | Task | Commit | Key Changes |
|---|------|--------|-------------|
| 1 | Add parsing dependencies | bce6def | Added calcard@0.3, rrule@0.14 to Cargo.toml |
| 2 | Implement iCal parser with RRULE expansion | d2cc387 | Created parser.rs, exported parse_ical |

## Implementation Details

### parse_ical Function

```rust
pub fn parse_ical(
    ical_data: &str,
    feed_url: &str,
    range_start: NaiveDate,
    range_end: NaiveDate,
    feed_name: Option<String>,
    feed_color: Option<String>,
) -> Result<Vec<CalendarEvent>>
```

**Key behaviors:**
- Parses VEVENT components from iCal data
- Extracts UID, SUMMARY, DTSTART, DTEND properties
- Detects all-day events (no time component in DTSTART)
- Expands RRULE recurrence rules within specified date range
- Safety cap of 100 occurrences per RRULE
- Returns CalendarEvent structs ready for database caching

### Helper Functions

| Function | Purpose |
|----------|---------|
| get_text_property | Extract text values from ICalendarEntry |
| parse_dtstart | Parse DTSTART with all-day detection |
| parse_dtend | Parse optional DTEND |
| parse_datetime_value | Convert PartialDateTime to NaiveDate/NaiveTime |
| get_rrule_string | Extract RRULE from RecurrenceRule or Text value |
| expand_rrule | Expand RRULE within date range using rrule crate |
| ensure_rrule_prefix | Normalize RRULE string format |

### calcard API Notes

- Uses `ICalendarComponentType::VEvent` (not `Event`)
- Entry has `values: Vec<ICalendarValue>` (not `value`)
- `PartialDateTime` fields (year, month, day) are `Option<T>`
- Rust 2024 edition: no explicit `ref` in pattern matching when implicitly borrowing

## Deviations from Plan

None - plan executed exactly as written.

## Files Changed

| File | Change |
|------|--------|
| Cargo.toml | Added calcard, rrule dependencies |
| Cargo.lock | Updated with new dependency tree |
| src/calendar/parser.rs | **NEW** - 254 lines, iCal parsing implementation |
| src/calendar/mod.rs | Added parser module, exported parse_ical |

## Next Phase Readiness

**Ready for 03-04:** Parser is exported and can be called from fetcher integration.

**Integration point:**
```rust
use crate::calendar::parse_ical;

let events = parse_ical(&ical_data, feed_url, range_start, range_end, name, color)?;
```

## Success Criteria Met

- [x] parse_ical returns CalendarEvent structs from iCal data
- [x] All-day events have all_day=true, no start_time
- [x] Recurring events expanded within date range
- [x] Safety limit prevents infinite RRULE expansion
- [x] Build succeeds without errors

---
*Completed: 2026-02-08*

