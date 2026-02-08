---
phase: 04-system
plan: 01
subsystem: window-persistence
tags: [egui, config, toml, window-state]

dependency-graph:
  requires: [01-01, 01-03]  # AppPaths, Config infrastructure
  provides: [window-state-persistence]
  affects: [04-02]  # May interact with system tray hide/restore

tech-stack:
  added: []  # No new dependencies
  patterns:
    - Periodic state save in update() loop
    - WAYLAND_DISPLAY detection for platform behavior

key-files:
  created:
    - ~/.config/wdid/window_state.toml
  modified:
    - src/config/types.rs
    - src/config/mod.rs
    - src/paths.rs
    - src/main.rs
    - src/app.rs

decisions:
  - id: periodic-save
    choice: "Save every 5 seconds if changed"
    rationale: "on_exit may not have full viewport context; periodic save is more reliable"
  - id: wayland-detection
    choice: "Check WAYLAND_DISPLAY env var"
    rationale: "Wayland compositors control window position; only persist size there"

metrics:
  duration: ~15 minutes
  completed: 2026-02-08
---

# Phase 04 Plan 01: Window Persistence Summary

**One-liner:** Window size and position persisted to TOML, restored on startup with Wayland-aware position handling.

## What Was Built

1. **WindowState type** (`src/config/types.rs`)
   - Struct with optional width, height, x, y fields
   - Serialize/Deserialize for TOML persistence

2. **Load/save functions** (`src/config/mod.rs`)
   - `load_window_state()` - Returns default if file missing
   - `save_window_state()` - Writes TOML with error handling

3. **Path infrastructure** (`src/paths.rs`)
   - Added `window_state_file` to AppPaths
   - Points to `~/.config/wdid/window_state.toml`

4. **Startup loading** (`src/main.rs`)
   - Load window state before creating app
   - Apply size via `with_inner_size()`
   - Apply position via `with_position()` (X11 only)

5. **Periodic saving** (`src/app.rs`)
   - Check every 5 seconds in update()
   - Save only if state changed from last save
   - Skip position on Wayland (compositor controls it)

## Commits

| Hash | Type | Description |
|------|------|-------------|
| 46ea5b4 | feat | Add WindowState type and persistence |
| 94d51dc | feat | Load window state on startup |
| c004a9e | feat | Save window state periodically |

## Deviations from Plan

None - plan executed exactly as written.

## Technical Notes

- Used `std::time::Instant` for tracking save interval (5 seconds)
- Used `ctx.input(|i| i.viewport())` to get current window geometry
- `inner_rect` gives content size, `outer_rect` gives position (including decorations)
- Change detection compares all 4 fields before saving

## Next Phase Readiness

Ready for 04-02 (System Tray):
- Window state will need to be saved before hiding to tray
- Consider calling `save_window_state_if_changed()` on close/hide events
- System tray restore should respect saved geometry

