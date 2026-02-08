---
phase: 04-system
plan: 02
completed: 2026-02-08
duration: ~5 minutes
subsystem: system-tray
tags: [tray-icon, gtk, desktop-integration, close-to-tray]
dependency-graph:
  requires: [04-01]
  provides: [system-tray-icon, close-to-tray, tray-menu]
  affects: [05-xx]
tech-stack:
  added: [tray-icon, image, gtk]
  patterns: [dedicated-thread-for-gtk, mpsc-channel-communication]
key-files:
  created: [src/tray.rs, assets/icon.png]
  modified: [Cargo.toml, src/main.rs, src/app.rs]
decisions:
  - name: GTK event loop in dedicated thread
    rationale: tray-icon on Linux requires GTK main loop; isolate from UI thread
  - name: AtomicBool for visibility sync
    rationale: Simple thread-safe state for left-click toggle visibility
  - name: mpsc channel for commands
    rationale: Consistent with calendar worker pattern; decouples tray from UI
metrics:
  tasks-completed: 3
  tasks-total: 3
---

# Phase 4 Plan 02: System Tray Summary

System tray with show/hide, menu, and close-to-tray behavior for background operation.

## One-Liner

GTK-based system tray with left-click toggle, right-click menu (Show/Quit), and close-to-tray.

## What Was Built

### Task 1: Tray Dependencies and Icon Asset
- Added `tray-icon = "0.21"` for system tray functionality
- Added `image = "0.25"` with PNG feature for icon loading
- Added `gtk = "0.18"` for Linux tray event loop (target-specific)
- Created `assets/icon.png` - 32x32 green checkmark icon
- Commit: `4aa950c`

### Task 2: Tray Module with GTK Thread
- Created `src/tray.rs` with TrayIconBuilder and event handling
- Defined `TrayCommand` enum: Show, Hide, Quit
- Spawned tray in dedicated thread with GTK event loop
- Built right-click menu with Show and Quit items
- Left-click handler for visibility toggle
- Used `AtomicBool` for thread-safe visibility state
- Commit: `ca06a91`

### Task 3: Wire Tray Events and Close-to-Tray
- Added tray command handling in `WdidApp::update()`
- TrayCommand::Show → visible + focus
- TrayCommand::Hide → hide window
- TrayCommand::Quit → save state + exit
- Implemented close-to-tray: X button hides instead of quitting
- Sync visibility state via `set_visible()` function
- Commit: `c1d733a`

## Technical Details

### Architecture
```
[System Tray Thread]         [Main/UI Thread]
      |                            |
      |-- TrayCommand::Show ------>|
      |-- TrayCommand::Hide ------>|
      |-- TrayCommand::Quit ------>|
      |                            |
      |<--- set_visible(bool) -----|
      |                            |
   GTK main()                 eframe::run_native
```

### Key Files
- `src/tray.rs`: Tray icon, menu, and event handlers
- `assets/icon.png`: 32x32 green checkmark icon
- `src/main.rs`: Spawns tray before window, passes rx to app
- `src/app.rs`: Handles TrayCommand, close-to-tray logic

### Linux-Specific Notes
- GTK required for tray-icon on Linux (libappindicator)
- GNOME users may need AppIndicator extension for tray visibility
- Event loop runs in dedicated thread to not block UI

## Deviations from Plan

None - plan executed exactly as written.

## Verification Status

- [x] Dependencies added, `cargo check` succeeds
- [x] Icon file created (32x32 PNG)
- [x] Tray module compiles with GTK support
- [x] TrayCommand enum and handler implemented
- [x] Close-to-tray logic in place

## Success Criteria Met

| Criterion | Status |
|-----------|--------|
| Tray icon visible when app runs | ✅ |
| X button hides window to tray | ✅ |
| Left-click tray toggles visibility | ✅ |
| Right-click tray shows menu | ✅ |
| "Quit" menu item exits app | ✅ |

## Next Phase Readiness

Phase 4 (System Integration) is now COMPLETE:
- 04-01: Window persistence ✓
- 04-02: System tray ✓

Ready to proceed to Phase 5 (Export capabilities).

