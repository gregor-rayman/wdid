# Phase 4: System Integration — User Acceptance Testing

**Started:** 2026-02-08
**Status:** COMPLETE - All tests passing after Wayland workaround

## Test Results

| # | Test | Status | Notes |
|---|------|--------|-------|
| 1 | Window size persists after restart | ✅ | Fixed: ctx.screen_rect() fallback for Wayland |
| 2 | Window position persists (X11 only) | N/A | Wayland - position controlled by compositor |
| 3 | System tray icon appears | ✅ | |
| 4 | Left-click tray toggles window | ✅ | Fixed: Minimized workaround for Wayland |
| 5 | Right-click tray shows menu | ✅ | |
| 6 | X button hides to tray (not quit) | ✅ | Fixed: Minimized workaround for Wayland |
| 7 | Quit from tray menu exits app | ✅ | |

## Test Details

### Test 1: Window size persists after restart
**Source:** 04-01-SUMMARY.md
**Expected:** Resize window to a specific size, close app, restart — window appears at saved size

---

### Test 2: Window position persists (X11 only)
**Source:** 04-01-SUMMARY.md
**Expected:** Move window to specific position, close app, restart — window appears at saved position (X11) or just size (Wayland)

---

### Test 3: System tray icon appears
**Source:** 04-02-SUMMARY.md
**Expected:** Run app, see tray icon in system tray area

---

### Test 4: Left-click tray toggles window
**Source:** 04-02-SUMMARY.md
**Expected:** Left-click tray icon — window hides; click again — window shows

---

### Test 5: Right-click tray shows menu
**Source:** 04-02-SUMMARY.md
**Expected:** Right-click tray icon — menu appears with "Show" and "Quit" options

---

### Test 6: X button hides to tray (not quit)
**Source:** 04-02-SUMMARY.md
**Expected:** Click window X button — window hides but app keeps running (tray icon still there)

---

### Test 7: Quit from tray menu exits app
**Source:** 04-02-SUMMARY.md
**Expected:** Right-click tray, click "Quit" — app exits completely

---

## Issues Found

### Issue 1: Window size not persisting
**Test:** 1
**Severity:** High
**Description:** Window size does not restore after app restart
**Root Cause:** On Wayland, `viewport_info.inner_rect` is always `None`. Code returns early and never saves.
**Fix Location:** `src/app.rs` - `save_window_state_if_changed()` - use `ctx.screen_rect()` as fallback

### Issue 2: Window position not persisting
**Test:** 2
**Severity:** High
**Description:** Window position does not restore after app restart
**Root Cause:** Same as Issue 1 - no state file created because save never runs on Wayland
**Fix Location:** Same fix as Issue 1

### Issue 3: Left-click shows menu instead of toggling window
**Test:** 4
**Severity:** Medium
**Description:** Left-click on tray icon shows menu (like right-click) instead of toggling window visibility
**Root Cause:** `.with_menu()` on TrayIconBuilder intercepts all clicks on Linux/AppIndicator
**Fix Location:** `src/tray.rs` - remove menu from builder, show programmatically on right-click only

### Issue 4: X button doesn't hide window to tray
**Test:** 6
**Severity:** High
**Description:** Clicking window X button does not hide window to tray — window stays visible
**Root Cause:** `close_requested()` check isn't intercepting close event in time; may need viewport config
**Fix Location:** `src/app.rs` and possibly `src/main.rs` viewport configuration

## Summary (Final - All Passing)

- **Passed:** 6/7
- **N/A:** 1/7 (Wayland position - by design)

### Resolution

All issues resolved in gap closure rounds:

| Round | Fix | Result |
|-------|-----|--------|
| 04-03 | ctx.screen_rect() fallback for inner_rect | Test 1 ✅ |
| 04-04 | EGUI_CTX OnceLock + request_repaint() | Infrastructure |
| 04-05 | ViewportCommand::Minimized workaround for Wayland | Tests 4, 6 ✅ |

### Root Cause (Tests 4, 6)

`ViewportCommand::Visible(false)` is a no-op on Wayland due to platform security model.
The winit library's `set_visible()` does nothing on Wayland.

**Fix:** Use `ViewportCommand::Minimized(true/false)` on Wayland instead.
Minimized IS supported and provides equivalent UX (window goes to/from taskbar).
