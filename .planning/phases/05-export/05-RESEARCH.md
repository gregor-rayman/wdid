# Phase 5: Export & Polish - Research

**Researched:** 2026-02-08
**Domain:** Export functionality (Markdown, JSON, clipboard, file dialogs)
**Confidence:** HIGH

## Summary

Phase 5 adds export functionality for diary entries to Markdown and JSON formats, plus standup and weekly retrospective summaries. The Rust ecosystem has mature, well-maintained crates for all required functionality.

The project already has `serde = "1.0.228"` with derive features. serde_json is a transitive dependency via calcard but should be added explicitly. For clipboard operations, arboard (maintained by 1Password) is the standard choice. For file save dialogs, rfd (Rusty File Dialogs) provides cross-platform native dialogs.

**Primary recommendation:** Add serde_json, arboard, and rfd dependencies. Create an export module with separate formatters for Markdown/JSON. Use egui's MenuBar for export triggers, offering both clipboard and file save options.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde_json | ~1.0 | JSON serialization | De facto standard, 100M+ downloads |
| arboard | ~3.4 | Clipboard access | 1Password maintained, cross-platform |
| rfd | ~0.15 | File save dialogs | Standard for Rust GUI apps, native dialogs |

### Already Present
| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| serde | 1.0.228 | Serialization derives | Already in Cargo.toml with derive feature |

**Installation:**
```bash
cargo add serde_json arboard --features arboard/wayland-data-control
cargo add rfd
```

## Architecture Patterns

### Recommended Module Structure
```
src/
├── export/
│   ├── mod.rs           # ExportConfig, ExportAction enum, public API
│   ├── markdown.rs      # Markdown formatting
│   ├── json.rs          # JSON serialization
│   └── summary.rs       # Standup/weekly summary generation
```

### Pattern 1: ExportAction Enum
Follow existing HeaderAction pattern for triggering exports:
```rust
pub enum ExportAction {
    ExportDayMarkdown,
    ExportDayJson,
    ExportStandup,
    ExportWeeklyRetro,
}
```

### Pattern 2: Menu Bar for Export
Use egui's MenuBar with MenuButton for export triggers:
```rust
// Source: https://docs.rs/egui/0.33.3/egui/containers/menu/struct.MenuBar.html
MenuBar::new(|ui| {
    MenuButton::new("Export").show(ui, |ui| {
        if ui.button("Today → Clipboard (Markdown)").clicked() { /* ... */ }
        if ui.button("Today → File (JSON)").clicked() { /* ... */ }
        SubMenuButton::new("Weekly Summary").show(ui, |ui| {
            if ui.button("Standup").clicked() { /* ... */ }
            if ui.button("Retrospective").clicked() { /* ... */ }
        });
    });
});
```

### Anti-Patterns to Avoid
- **Direct serde_json in UI code:** Keep serialization in export module
- **Blocking file dialogs:** rfd::FileDialog is sync but fast; acceptable for save dialogs

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON serialization | Custom formatters | serde_json::to_string_pretty | Edge cases, escaping, spec compliance |
| Clipboard access | Platform-specific code | arboard::Clipboard | X11/Wayland/Win/Mac differences |
| File save dialogs | Custom dialog UI | rfd::FileDialog | Native OS dialogs expected by users |
| Date range queries | Multiple single-date queries | New DB method | Performance, atomicity |

## Common Pitfalls

### Pitfall 1: Missing Serialize Derives
**What goes wrong:** DiaryEntry and CalendarEvent lack #[derive(Serialize)]
**Why it happens:** Structs were created before export was needed
**How to avoid:** Add `#[derive(serde::Serialize)]` to DiaryEntry, CalendarEvent
**Warning signs:** Compiler errors about Serialize trait not implemented

### Pitfall 2: No Date Range Query
**What goes wrong:** Weekly export requires 7 separate DB calls
**Why it happens:** Only `get_entries_for_date()` exists currently
**How to avoid:** Add `get_entries_for_date_range(start: &str, end: &str)` to Database
**Warning signs:** Slow weekly exports, N+1 query pattern

### Pitfall 3: Linux Clipboard Ownership
**What goes wrong:** Clipboard content disappears when app closes
**Why it happens:** On Linux, clipboard is "served" by the owning app
**How to avoid:** arboard handles this correctly; just be aware of the behavior
**Warning signs:** Users report clipboard empty after closing app

### Pitfall 4: Wayland Clipboard Support
**What goes wrong:** Clipboard doesn't work on Wayland
**Why it happens:** Missing wayland-data-control feature
**How to avoid:** Enable `arboard/wayland-data-control` feature in Cargo.toml
**Warning signs:** Clipboard operations fail silently on Wayland systems

## Code Examples

### JSON Export with serde_json
```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct ExportEntry {
    pub date: String,
    pub start_time: String,
    pub duration: Option<i32>,
    pub content: String,
    pub event_snapshot: Option<String>,
}

pub fn export_json(entries: &[DiaryEntry]) -> String {
    serde_json::to_string_pretty(&entries).unwrap_or_default()
}
```

### Clipboard with arboard
```rust
use arboard::Clipboard;

pub fn copy_to_clipboard(text: &str) -> Result<(), arboard::Error> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}
```

### File Save with rfd
```rust
use rfd::FileDialog;

pub fn save_to_file(content: &str, default_name: &str) -> Option<()> {
    let path = FileDialog::new()
        .set_file_name(default_name)
        .add_filter("Markdown", &["md"])
        .add_filter("JSON", &["json"])
        .save_file()?;
    std::fs::write(&path, content).ok()
}
```

## Export Format Recommendations

### Markdown Export Format
```markdown
# 2026-02-08 (Saturday)

## 09:00 - 10:00 (1h) - Team Standup
Discussed sprint progress. #meeting

## 10:30 - 12:00 (1h 30m)
Worked on export feature implementation. #coding
```

### Standup Summary Format (concise, today only)
```markdown
**Yesterday:**
- Team standup (1h)
- Export feature implementation (1h 30m)

**Today:**
- Continue export work
- Code review
```

### Weekly Retro Format (grouped by day with totals)
```markdown
# Week of 2026-02-03

## Monday (6h 30m)
- Sprint planning (2h) #meeting
- Feature development (4h 30m) #coding

## Tuesday (7h 15m)
...

**Weekly Total:** 32h 45m
```

## Open Questions

1. **Keyboard shortcuts for export?**
   - Recommendation: Cmd/Ctrl+E for quick export menu, let menu handle specifics

2. **Date range picker UI?**
   - Recommendation: Start with "This Week" preset; defer custom range to future

## Sources

### Primary (HIGH confidence)
- serde_json docs.rs - serialization API
- arboard GitHub README - clipboard usage, Wayland support
- rfd docs.rs - file dialog API
- egui 0.33.3 docs - MenuBar, MenuButton, SubMenuButton

### Codebase Analysis (HIGH confidence)
- src/db/entries.rs - DiaryEntry struct, existing queries
- src/calendar/types.rs - CalendarEvent struct
- Cargo.toml - existing serde dependency
- src/ui/header.rs - HeaderAction pattern

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - well-established crates, official docs verified
- Architecture: HIGH - follows existing codebase patterns
- Pitfalls: HIGH - identified from docs and codebase analysis

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (stable libraries, 30-day validity)

