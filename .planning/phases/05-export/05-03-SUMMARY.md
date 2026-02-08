---
phase: 05-export
plan: 03
subsystem: export-ui
tags: [export, menu, clipboard, file-dialog, ui]
dependency-graph:
  requires: [05-02]
  provides: [export-menu, export-actions, clipboard-copy, file-save]
  affects: []
tech-stack:
  added: []
  patterns: [menu-button, action-tuple-return, clipboard-api, file-dialog-api]
key-files:
  created: []
  modified:
    - src/ui/header.rs
    - src/app.rs
    - src/export/mod.rs
decisions:
  - key: menu-button-pattern
    choice: egui::menu::menu_button with close_menu()
    rationale: Standard egui pattern for dropdown menus
  - key: action-return-type
    choice: Tuple (HeaderAction, ExportAction) from render_header
    rationale: Follows existing pattern, keeps header stateless
  - key: error-handling
    choice: eprintln for clipboard errors, silent skip for file cancel
    rationale: Non-blocking UX, errors logged for debugging
---

# Summary: Export UI & Wiring

## What Was Built

Added Export menu to header with full clipboard and file export functionality:

### Export Menu (6 options)
- 📋 Today → Clipboard (Markdown)
- 💾 Today → File (Markdown)
- 📋 Today → Clipboard (JSON)
- 💾 Today → File (JSON)
- 📋 Standup Summary
- 📋 Weekly Retro

### Implementation Details

1. **Header Changes** (`src/ui/header.rs`)
   - Added `ExportAction` import from export module
   - Changed `render_header()` return type to `(HeaderAction, ExportAction)`
   - Added Export menu button with `egui::menu::menu_button`
   - All 6 export options with appropriate icons

2. **Export Helpers** (`src/export/mod.rs`)
   - `copy_to_clipboard(text)` - Uses arboard for cross-platform clipboard
   - `save_to_file(content, name, filter_name, filter_ext)` - Uses rfd for native dialogs

3. **Action Handler** (`src/app.rs`)
   - Added `handle_export_action()` method to WdidApp
   - Handles all 6 ExportAction variants
   - Weekly retro calculates Monday-Sunday range and queries date range

## Tasks Completed

| # | Task | Commit | Status |
|---|------|--------|--------|
| 1 | Add Export menu to header | f539f27 | ✅ |
| 2 | Wire export actions in app.rs | c5fcd3c | ✅ |
| 3 | Human verification checkpoint | — | ✅ approved |

## Verification

User verified:
- Export menu appears in header with 6 options ✓
- Clipboard copy works for Markdown, JSON, Standup, Weekly Retro ✓
- File save dialog opens for Markdown and JSON exports ✓

## Commits

| Hash | Type | Description |
|------|------|-------------|
| f539f27 | feat | Add Export menu to header |
| c5fcd3c | feat | Wire export actions in app.rs |

## Requirements Addressed

- **EXP-01**: Export to Markdown (clipboard + file)
- **EXP-02**: Export to JSON (clipboard + file)
- **EXP-03**: Standup summary (clipboard)
- **EXP-04**: Weekly retro (clipboard)

