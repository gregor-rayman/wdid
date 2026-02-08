# Phase 3 Context: Calendar Integration

**Created:** 2026-02-08
**Phase Goal:** Users can see their calendar events alongside diary entries in a unified timeline.

---

## Event Display & Density

### All-Day Events
- Display at the **top of the day**, above the timeline
- Separate section from timed events

### Overlapping Events
- Show **side-by-side** within the column (narrower but compact)
- No stacking or hiding

### Long Events (multi-hour)
- Display as **compact card with duration text** (e.g., "9:00-13:00")
- Not proportional height — keeps UI compact

### High Event Volume
- **Show all events**, let the column scroll independently
- No collapsing or "show more" — accept density

---

## Feed Configuration & Identity

### Visual Distinction
- **Color-coded** per feed (left border or background tint)
- Each feed configured with its own color in config.toml

### Color Configuration
- User specifies color per feed **in config.toml**
- No auto-assignment — explicit user control

### Network Errors (unreachable feed)
- Show **last cached events** with a subtle "stale" indicator
- No error banner — graceful degradation

### Invalid Feed (malformed URL or bad iCal data)
- Show **persistent error in UI** until user fixes config
- Don't silently skip — make the problem visible

---

## Linking Diary Entries to Events

### Initiating a Link
- **Both directions supported:**
  - Click "add note" button on calendar event → creates new linked entry
  - Right-click existing diary entry → "Link to event" → pick from event list

### Visual Indicator
- Linked entries show **colored left border** matching the calendar feed color
- No title badge — border is sufficient

### Unlinking
- **Yes, via right-click context menu** "Unlink from event"
- Clean separation from delete

### Orphaned Links (event deleted from source)
- Entry **remains**, link becomes orphaned
- Show message: "Event no longer exists"
- User decides whether to keep or delete entry

---

## Time Alignment & Scrolling

### Row Layout
- **Dynamic rows based on content**
- Items appear at their time, empty gaps compressed
- No fixed time slots

### Scroll Behavior
- **Locked together** — scrolling moves both columns in sync
- Single scroll context for unified experience

### Initial Scroll Position
- Open at **current time** (now) centered or near top
- User sees "now" immediately

### Time Markers
- **Subtle hour markers in the gap** between columns
- Not spanning full width — unobtrusive

---

## Config.toml Structure

```toml
[[calendars]]
name = "Work"
url = "https://calendar.example.com/work.ics"
color = "#4A90D9"

[[calendars]]
name = "Personal"
url = "https://calendar.example.com/personal.ics"
color = "#7CB342"
```

- Array of calendar tables
- Each has: name, url, color
- 3-5 feeds supported per requirements

---

## Deferred Ideas

*(Captured during discussion, not in scope for Phase 3)*

- None identified

---

## Open Questions for Research

1. Which iCal parsing crate handles edge cases best? (calcard vs icalendar)
2. How to cache feed data for offline/stale display?
3. egui layout approach for two synchronized scroll areas?
4. How to detect "stale" state (last successful refresh timestamp)?

---
*Context created: 2026-02-08*

