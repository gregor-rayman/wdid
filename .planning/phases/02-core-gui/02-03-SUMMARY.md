---
phase: 02-core-gui
plan: 03
subsystem: ui
tags: [editing, egui, timeline, context-menu]
dependency-graph:
  requires: [02-02]
  provides: [entry-editing, context-menu-delete, duration-editing]
  affects: [02-04, 02-05]
tech-stack:
  added: []
  patterns: [dual-mode-render, action-enum]
key-files:
  created: []
  modified:
    - src/ui/entry.rs
    - src/ui/state.rs
    - src/ui/timeline.rs
    - src/app.rs
    - src/db/entries.rs
decisions:
  - id: entry-action-enum
    choice: "EntryAction enum for Save/Delete actions"
    rationale: "Clean separation between UI rendering and state mutation"
  - id: dual-mode-render
    choice: "render_entry function with is_editing check"
    rationale: "Single entry point for both view and edit modes"
  - id: save-triggers
    choice: "Escape key or lost focus triggers save"
    rationale: "Matches common inline editing patterns"
  - id: delete-on-empty
    choice: "Empty content triggers delete instead of save"
    rationale: "Natural way to remove entries without explicit delete"
metrics:
  duration: 5m
  completed: 2026-02-08
---

# Phase 2 Plan 3: Entry Editing Summary

**One-liner:** Dual-mode entry rendering with click-to-edit, Escape-to-save, context menu delete, and duration editing.

## What Was Built

### Core Entry Editing System
- **EntryAction enum**: `None | Save { id, content, start_time, duration } | Delete(id)` for clean action handling
- **Dual-mode render_entry()**: Dispatches to edit or view mode based on `editing_entry_id`
- **Edit mode**: TextEdit::multiline for content, TextEdit::singleline for time (HH:MM) and duration (minutes)
- **View mode**: CommonMarkViewer for markdown, clickable frame to enter edit mode

### Save and Delete Mechanics
- **Save triggers**: Escape key press OR lost focus (click outside)
- **Focus management**: Auto-focus on first frame via `edit_focus_set` flag
- **Delete-on-empty**: Saving with empty content deletes entry automatically
- **Context menu delete**: Right-click → "🗑 Delete" button

### Database Support
- **update_entry_full()**: Updates content, start_time, and duration in single call

### State Management
- Added to DiaryViewState:
  - `start_time_buffer`: HH:MM string for time editing
  - `duration_buffer`: Minutes as string for duration editing
  - `edit_focus_set`: Boolean to track initial focus

### Timeline Integration
- **TimelineActions struct**: Collects save/delete actions from entry rendering
- **App update loop**: Processes actions, handles delete-on-empty, reloads entries

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Action pattern | EntryAction enum | Separates rendering from mutation, testable |
| Save trigger | Escape OR lost_focus | Common inline edit pattern |
| Delete mechanism | Empty content OR context menu | Two natural paths to same action |
| Focus tracking | edit_focus_set boolean | Prevents focus request every frame |

## Files Modified

| File | Changes |
|------|---------|
| src/ui/entry.rs | EntryAction enum, render_entry, render_edit_mode, render_view_mode with context menu |
| src/ui/state.rs | start_time_buffer, duration_buffer, edit_focus_set fields |
| src/ui/timeline.rs | TimelineActions struct, render_timeline returns actions |
| src/app.rs | Handle timeline actions, delete-on-empty, reload entries |
| src/db/entries.rs | update_entry_full method |

## Verification Results

| Check | Status |
|-------|--------|
| `cargo check` | ✅ No errors (6 warnings for future-phase fields) |
| Click entry → edit mode | ✅ Implemented |
| Escape → save | ✅ Implemented |
| Click outside → save | ✅ Implemented (lost_focus) |
| Right-click → context menu | ✅ Implemented |
| Delete via menu | ✅ Implemented |
| Clear text → delete | ✅ Implemented |
| Duration editing | ✅ Implemented |

## Requirements Satisfied

- ✅ **DIARY-02**: User can edit entry in place (raw markdown input, rendered display)
- ✅ **DIARY-03**: User can delete entry via right-click context menu
- ✅ **DIARY-04**: User can delete entry by clearing all text
- ✅ **DIARY-07**: User can set optional duration on diary entry

## Commits

| Hash | Message |
|------|---------|
| 28b095e | feat(02-03): implement click-to-edit and save behavior |

## Deviations from Plan

None - plan executed exactly as written. Task 2 items were implemented together with Task 1 as they were tightly coupled.

## Next Phase Readiness

### Provides for 02-04 (Settings Panel)
- Entry editing patterns established
- State management patterns in place

### Provides for 02-05 (Search)
- DiaryViewState has search_query and search_results fields ready

### No Blockers
- All success criteria met
- Clean compile with only future-phase warnings

