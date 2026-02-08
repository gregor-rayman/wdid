# Phase 2 Context: Core GUI & Diary

**Created:** 2026-02-08
**Phase Goal:** Users can create, edit, and organize diary entries in a navigable daily view.

---

## Entry Creation Flow

### How entries are created
- **Timeline click**: Click at a time position → creates entry at that time (snapped to 15-minute intervals)
- **Keyboard shortcut**: `Ctrl+n` → creates entry at current time (rounded to nearest 15 min)
- Time is always editable after creation

### Time snapping
- All times snap to 15-minute intervals: :00, :15, :30, :45
- User can manually edit start time to any value after creation

---

## Timeline Layout & Density

### Entry arrangement
- **Stacked/compact layout**: Entries listed in time order, not proportional to clock time
- **Visual gaps**: 1-line gap between entries when time gap ≥ 30 minutes during work hours
- **Outside work hours**: No visual gaps, entries stack directly

### Work hours (configurable)
- Default: 09:00–17:00, Monday–Saturday
- Configurable in config.toml: `work_hours_start`, `work_hours_end`, `work_days`

### Entry sizing
- **Content-sized with max**: Entry grows to fit content, up to N lines
- Default max lines: 8 (configurable in config.toml)
- Entries exceeding max lines scroll internally

### Time indicators
- Each entry displays its start time and end time (if duration set)
- **"Now" indicator**: Subtle background highlight on current time position (today's view only)

---

## Edit/View Mode Behavior

### Switching modes
- **Click to edit**: Single click on entry → switches to raw markdown edit mode
- **Click away to save**: Click outside entry → saves and renders markdown
- **Escape to save**: Press Esc → saves and exits edit mode
- **Undo**: Ctrl+Z works during editing

### Editing location
- **Inline editing**: Entry expands in place within the timeline
- No modal or side panel — edit right where the entry lives

### Markdown rendering
- **Full rendering** in view mode: headers, bold, italic, lists, code blocks, links
- Hashtags (#tag) are visually distinct and clickable

---

## Search & Results Display

### Search box location
- **Top of window**: Always visible in header/toolbar area
- Persistent — doesn't scroll with content

### Search syntax
- **Prefix-based matching**:
  - `#project-x` → searches hashtags only
  - `meeting` → searches full text content
- Both can be used; prefix determines search type

### Results display
- **Replaces timeline**: Search results take over main view
- Shows matching entries from all dates
- Entries displayed with their dates visible

### Returning to timeline
- **Clear search box**: Delete text or click X button → returns to today's view
- No explicit back button needed

---

## Deferred Ideas

*(Captured during discussion but out of scope for Phase 2)*

None captured.

---

## Requirements Covered

| Requirement | How it's addressed |
|-------------|-------------------|
| DIARY-01 | Timeline click + Ctrl+n, 15-min snapping |
| DIARY-02 | Click to edit inline, full markdown rendering |
| DIARY-03 | Right-click context menu (implementation detail) |
| DIARY-04 | Clear all text → deletes entry |
| DIARY-07 | Duration shown as end time on entry |
| DIARY-08 | Hashtags rendered distinctly, searchable with # prefix |
| NAV-01 | Arrow buttons for day navigation |
| NAV-02 | Single-day view, today by default |
| SRCH-01 | #tag search across all dates |
| SRCH-02 | Search box in header, results replace timeline |

---
*Context captured: 2026-02-08*

