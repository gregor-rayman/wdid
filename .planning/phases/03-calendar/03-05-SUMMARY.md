---
phase: 03-calendar
plan: 05
subsystem: ui-timeline
tags: [egui, two-column, layout, scrolling, calendar-events, diary-entries]
completed: 2026-02-08
duration: ~5 minutes

requires:
  - 03-04: Pipeline integration (calendar_events, all_day_events in state)
provides:
  - Two-column timeline layout (calendar left, diary right)
  - Synchronized scroll between columns
  - All-day events rendered above columns
  - Calendar column renderer with color-coded borders
affects:
  - 03-06: Event interactions (click handling on calendar events)
  - 03-07: Header enhancements (calendar refresh status)

tech-stack:
  added: []
  patterns:
    - Two-column layout with ui.columns(2, ...)
    - Synchronized scrolling via shared scroll_offset
    - Hover-based scroll priority detection
    - Color-coded event cards with left border

key-files:
  created:
    - src/ui/calendar_column.rs
  modified:
    - src/ui/state.rs
    - src/ui/timeline.rs
    - src/ui/mod.rs
    - src/app.rs
---

# Phase 3 Plan 05: Two-Column UI Summary

**One-liner:** Two-column timeline with calendar events left, diary entries right, synchronized scrolling via hover detection.

## What Was Done

### Task 1: Add scroll state to DiaryViewState

Added scrolling and column tracking fields to `DiaryViewState`:
- `Column` enum with `Calendar` and `Diary` variants for tracking active column
- `scroll_offset: f32` - shared y-axis scroll position for sync scrolling
- `hovered_column: Option<Column>` - tracks which column user is interacting with

### Task 2: Create calendar column renderer

Created `src/ui/calendar_column.rs` with:
- `render_all_day_events(ui, events)` - horizontal wrapped display with chip-style badges
- `render_calendar_events(ui, events)` - vertical card list for timed events
- `render_event_card(ui, event)` - individual event with color-coded left border (4px)
- `parse_color(hex)` - hex string (#RRGGBB) to Color32 conversion
- Unit tests for color parsing edge cases

Updated `src/ui/mod.rs` to export the new module.

### Task 3: Implement two-column timeline with sync scroll

Refactored `render_timeline()`:
- Added `calendar_events` and `all_day_events` parameters
- Two-column layout using `ui.columns(2, ...)`
- Left column: calendar events via `render_calendar_events()`
- Right column: diary entries via new `render_diary_entries()` helper
- All-day events rendered at top above both columns
- Synchronized scrolling via shared `scroll_offset` in state
- Hover-based scroll priority detection
- Single-column layout preserved for search mode
- Empty state handling for no events/entries

Updated call site in `src/app.rs` to pass calendar events using `std::mem::take()` pattern to avoid borrow conflicts.

## Commits

| Hash | Message |
|------|---------|
| e5d27de | feat(03-05): add scroll state to DiaryViewState |
| f1de35e | feat(03-05): create calendar column renderer |
| 5052813 | feat(03-05): implement two-column timeline with sync scroll |

## Verification

- [x] `cargo build` compiles without errors
- [x] `cargo test` passes (3 tests for color parsing)
- [x] Calendar events display in left column of timeline
- [x] Diary entries display in right column
- [x] Both columns scroll in sync via shared offset
- [x] All-day events appear at top above scrollable columns
- [x] Search mode shows single-column layout

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed deprecated egui API usage**
- **Found during:** Task 2 (calendar column renderer)
- **Issue:** `Rounding` type deprecated in favor of `CornerRadius`, `Frame::none()` deprecated
- **Fix:** Used `CornerRadius::same(N)` with u8 values, `Frame::new()` with `.corner_radius()`
- **Files modified:** src/ui/calendar_column.rs
- **Commit:** f1de35e

## Next Phase Readiness

Ready for 03-06 (Event Interactions):
- Calendar events visible in left column with color-coded borders
- All-day events displayed as chips at top
- Event cards ready for click handler addition
- Scroll state tracked for potential scroll-to-event functionality

