---
phase: 05-export
plan: 02
subsystem: export
tags: [markdown, json, standup, retro, formatting]
dependency-graph:
  requires: [05-01]
  provides: [export-formatters, markdown-export, json-export, summary-generation]
  affects: [05-03, 05-04]
tech-stack:
  added: []
  patterns: [format-functions, duration-formatting]
key-files:
  created: []
  modified:
    - src/export/markdown.rs
    - src/export/json.rs
    - src/export/summary.rs
    - src/export/mod.rs
decisions:
  - key: event-snapshot-parsing
    choice: Split on ':' to extract summary from "color:summary" format
    rationale: Matches existing snapshot format from 03-07
  - key: standup-truncation
    choice: Truncate first line to 80 chars
    rationale: Keeps standup messages concise and focused
  - key: BTreeMap-for-weekly
    choice: BTreeMap groups entries by date in sorted order
    rationale: Days appear chronologically in weekly retro output
metrics:
  duration: ~2 minutes
  completed: 2026-02-08
---

# Phase 5 Plan 02: Export Formatters Summary

**One-liner:** Markdown, JSON, and summary formatters for export (format_day_markdown, format_entries_json, format_standup, format_weekly_retro).

## What Was Built

Implemented all export formatting functions:

1. **Markdown formatter (src/export/markdown.rs):**
   - `format_day_markdown(date, entries)` - Creates date-headed Markdown document
   - `format_entry_markdown(entry)` - Formats individual entries with time, duration, event name
   - Helper functions: `format_duration()`, `add_minutes_to_time()`

2. **JSON formatter (src/export/json.rs):**
   - `format_entries_json(entries)` - Pretty-printed JSON using serde_json

3. **Summary formatters (src/export/summary.rs):**
   - `format_standup(entries)` - Concise bullet list for daily standups
   - `format_weekly_retro(entries, week_start)` - Grouped by day with totals

4. **Module exports (src/export/mod.rs):**
   - All public functions re-exported for easy access

## Commits

| Hash | Description |
|------|-------------|
| 0df2ccc | feat(05-02): implement Markdown formatter |
| 0a6d684 | feat(05-02): implement JSON and summary formatters |

## Key Code

### Markdown Day Format

```rust
pub fn format_day_markdown(date: &NaiveDate, entries: &[DiaryEntry]) -> String {
    // # 2026-02-08 (Saturday)
    // ## 09:00 - 10:30 (1h 30m) — Meeting Name
    // Entry content here
}
```

### Standup Format

```rust
pub fn format_standup(entries: &[DiaryEntry]) -> String {
    // **What I did:**
    // - Task 1 summary (30m)
    // - Task 2 summary (1h)
}
```

### Weekly Retro Format

```rust
pub fn format_weekly_retro(entries: &[DiaryEntry], week_start: &NaiveDate) -> String {
    // # Week of 2026-02-03
    // ## 2026-02-03 (Monday) — 4h 30m
    // - Task summaries...
    // **Weekly Total:** 32h 15m
}
```

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| BTreeMap for day grouping | Ensures chronological order in weekly retro |
| 80 char truncation for standup | Keeps standup summaries focused |
| Split(':').nth(1) for event summary | Matches existing "color:summary" snapshot format |

## Deviations from Plan

None - plan executed exactly as written.

## Next Phase Readiness

Plan 05-03 can proceed to:
- Wire formatters to clipboard/file output
- Add export UI menu using ExportAction enum

