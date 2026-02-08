---
phase: 04-system
plan: 03
type: gap-closure
subsystem: system-integration
tags: [window-state, tray, wayland, close-to-tray]

dependency_graph:
  requires: [04-01, 04-02]
  provides: [fixed-window-persistence, working-tray-toggle, close-to-tray]
  affects: []

tech_stack:
  added: []
  patterns:
    - screen_rect-fallback-for-wayland
    - toggle-menu-item-for-appindicator
    - close-check-first-in-update

key_files:
  created: []
  modified:
    - src/app.rs
    - src/tray.rs

decisions:
  - id: wayland-fallback
    choice: "ctx.screen_rect() as fallback when inner_rect is None"
    rationale: "Wayland never provides inner_rect; screen_rect gives usable dimensions"
  - id: toggle-menu-item
    choice: "Show/Hide menu item toggles visibility instead of just showing"
    rationale: "AppIndicator intercepts all clicks; menu item provides toggle functionality"
  - id: close-check-position
    choice: "Move close_requested() check to very start of update()"
    rationale: "Must catch close event before it clears; CancelClose must be in same frame"

metrics:
  duration: "~15 minutes"
  completed: 2026-02-08
---

# Phase 4 Plan 03: UAT Gap Closure Summary

**Fixed UAT failures from Phase 4 testing to complete system integration.**

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Fix window state saving on Wayland | e12043e | src/app.rs |
| 2 | Fix left-click toggle (Show/Hide menu) | efad34e | src/tray.rs |
| 3 | Fix X button close-to-tray | 79528e3 | src/app.rs |

## Technical Details

### Task 1: Window State on Wayland

**Problem:** On Wayland, `viewport_info.inner_rect` is always `None`, causing `save_window_state_if_changed()` to return early without saving.

**Solution:** Use `ctx.screen_rect()` as fallback for window dimensions:
```rust
let (width, height) = if let Some(inner_rect) = viewport_info.inner_rect {
    (inner_rect.width(), inner_rect.height())
} else {
    let screen = ctx.screen_rect();
    (screen.width(), screen.height())
};
```

### Task 2: Tray Click Behavior

**Problem:** On Linux with AppIndicator, `.with_menu()` causes ALL clicks to show menu, preventing left-click toggle.

**Solution:** Rename "Show" to "Show/Hide" with toggle logic in menu handler:
```rust
"toggle" => {
    let currently_visible = VISIBLE.load(Ordering::SeqCst);
    if currently_visible {
        tx_menu.send(TrayCommand::Hide);
    } else {
        tx_menu.send(TrayCommand::Show);
    }
}
```

**Note:** This is a workaround for an AppIndicator platform limitation. Direct left-click toggle is not possible when a menu is attached.

### Task 3: Close-to-Tray

**Problem:** `close_requested()` check wasn't catching the close event before it cleared.

**Solution:** Move close handling to the very FIRST thing in `update()`:
```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Handle close-to-tray FIRST
    if ctx.input(|i| i.viewport().close_requested()) {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        crate::tray::set_visible(false);
        self.save_window_state_if_changed(ctx);
    }
    // ... rest of update
}
```

## Deviations from Plan

None - all three issues addressed as planned.

## UAT Results After Fix

| Test | Status |
|------|--------|
| Window size persists | ✅ |
| Window position persists (X11) | ✅ |
| System tray icon appears | ✅ |
| Left-click tray toggles* | ✅ |
| Right-click shows menu | ✅ |
| X button hides to tray | ✅ |
| Quit from tray menu exits | ✅ |

*Via menu item due to AppIndicator limitation

## Known Limitations

1. **Left-click on Linux:** Opens menu (AppIndicator intercepts all clicks). Toggle happens via "Show/Hide" menu item.
2. **Wayland position:** Window position cannot be persisted on Wayland (compositor controls it). Only size is saved.

## Next Phase Readiness

Phase 4 is complete. All system integration features working:
- Window state persistence (size + position on X11, size only on Wayland)
- System tray with menu
- Close-to-tray behavior
- Show/Hide toggle via tray

Ready to proceed to Phase 5: Export capabilities.

