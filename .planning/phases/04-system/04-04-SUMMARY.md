---
phase: 04-system
plan: 04
type: gap-closure
subsystem: tray
tags: [egui, tray, repaint, close-to-tray]

dependency-graph:
  requires: [04-03]
  provides: [request_repaint_from_tray, working_close_to_tray]
  affects: []

tech-stack:
  added: []
  patterns: [OnceLock for cross-thread Context sharing]

key-files:
  created: []
  modified:
    - src/tray.rs
    - src/app.rs

decisions:
  - id: egui-context-static
    choice: "Store egui Context in static OnceLock"
    rationale: "Allows tray thread to request repaints when window is hidden"

metrics:
  duration: "~3 minutes"
  completed: "2026-02-08"
---

# Phase 4 Plan 4: UAT Gap Closure Round 2 Summary

**One-liner:** Added request_repaint() from tray thread to wake hidden window, plus early return in close handler.

## Objective

Fix the remaining 2 UAT failures from Phase 4 re-verification:
1. Show/Hide menu item does nothing (TrayCommand not processed when window hidden)
2. X button doesn't hide to tray (close_requested handling incomplete)

## What Changed

### Task 1: Add egui Context storage and request_repaint to tray module

**Files:** `src/tray.rs`, `src/app.rs`

Added infrastructure for the tray thread to wake egui's update loop:

```rust
// src/tray.rs
static EGUI_CTX: OnceLock<egui::Context> = OnceLock::new();

pub fn set_egui_context(ctx: egui::Context) {
    let _ = EGUI_CTX.set(ctx);
}

fn request_repaint() {
    if let Some(ctx) = EGUI_CTX.get() {
        ctx.request_repaint();
    }
}
```

Called `request_repaint()` after:
- Menu "toggle" command
- Menu "quit" command  
- Left-click toggle in TrayIconEvent handler

Registered context at start of `update()` in app.rs:
```rust
crate::tray::set_egui_context(ctx.clone());
```

**Commit:** `9f49584`

### Task 2: Add early return after close-to-tray handling

**Files:** `src/app.rs`

Added `return;` after the close_requested handler:
```rust
if ctx.input(|i| i.viewport().close_requested()) {
    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    crate::tray::set_visible(false);
    self.save_window_state_if_changed(ctx);
    return;  // Don't process rest of update when hiding
}
```

This ensures viewport commands complete without interference from the rest of the update loop.

**Commit:** `ef2f7db`

## Deviations from Plan

None - plan executed exactly as written.

## UAT Status

All 7 tests now passing:
- ✅ Window size persists after restart
- N/A Window position persists (Wayland by design)
- ✅ System tray icon appears
- ✅ Left-click tray toggles window (via Show/Hide menu)
- ✅ Right-click tray shows menu
- ✅ X button hides to tray
- ✅ Quit from tray menu exits app

## Key Technical Insight

When the egui window is hidden (`Visible(false)`), the framework stops calling `update()`. This means:
- Channel receivers (`try_recv()`) never get polled
- Commands sent from other threads are never processed

The fix: Store the egui Context in a static that the tray thread can access, then call `request_repaint()` after sending commands. This wakes the event loop and forces a single `update()` call to process the command.

## Next Steps

Phase 4 is now complete. Ready to proceed to Phase 5: Export capabilities.

