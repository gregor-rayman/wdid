# Phase 2: Core GUI & Diary - Research

**Researched:** 2026-02-08
**Domain:** egui GUI development, diary/timeline UI patterns
**Confidence:** HIGH

## Summary

Phase 2 implements the core diary interface using egui 0.32.0 with eframe. The research focused on egui patterns for timeline-based views, inline editing, markdown rendering, context menus, keyboard shortcuts, and state management for edit/view modes.

The immediate mode paradigm means UI state must be managed externally in application state - widgets don't retain state between frames. The TEA (The Elm Architecture) pattern from Phase 1 aligns well with egui's approach.

**Primary recommendation:** Use egui_commonmark 0.21 for markdown rendering (compatible with egui 0.32), chrono for date arithmetic, and implement edit mode via external state tracking with TextEdit widgets.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| egui | 0.32.0 | Immediate mode GUI | Already in project |
| eframe | 0.32.0 | egui framework | Already in project |
| egui_commonmark | 0.21 | Markdown rendering | Supports egui 0.32, full CommonMark |
| chrono | 0.4 | Date/time handling | Standard for Rust, already in use |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| pulldown-cmark | (via egui_commonmark) | Markdown parsing | Automatic with egui_commonmark |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| egui_commonmark | Custom rendering | Much more work, inconsistent styling |
| chrono | time crate | chrono has better date arithmetic |

**Installation:**
```bash
cargo add egui_commonmark@0.21
```

## Architecture Patterns

### Recommended State Structure
```rust
pub struct DiaryViewState {
    current_date: NaiveDate,
    editing_entry_id: Option<i64>,  // Which entry is being edited
    edit_buffer: String,            // Text being edited
    search_query: String,
    search_results: Option<Vec<DiaryEntry>>,
}
```

### Pattern 1: Click-to-Edit Implementation
**What:** Toggle between view and edit mode for entries
**When to use:** When user clicks entry text or presses Enter
**Example:**
```rust
// In entry rendering loop:
if let Some(editing_id) = state.editing_entry_id {
    if editing_id == entry.id {
        let response = ui.add(TextEdit::multiline(&mut state.edit_buffer));
        if response.lost_focus() || ui.input(|i| i.key_pressed(Key::Escape)) {
            // Save and exit edit mode
            save_entry_content(entry.id, &state.edit_buffer);
            state.editing_entry_id = None;
        }
    }
} else {
    // View mode - render markdown
    CommonMarkViewer::new().show(ui, &mut cache, &entry.content);
    if ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
        state.editing_entry_id = Some(entry.id);
        state.edit_buffer = entry.content.clone();
    }
}
```

### Pattern 2: Keyboard Shortcuts
**What:** Global keyboard handling with consume_key
**When to use:** For Ctrl+N (new entry), Escape (cancel), etc.
**Example:**
```rust
// At start of update(), before any widgets
ui.input(|i| {
    if i.consume_key(Modifiers::CTRL, Key::N) {
        // Create new entry at current time
    }
    if i.key_pressed(Key::Escape) && state.editing_entry_id.is_some() {
        state.editing_entry_id = None;
    }
});
```

### Pattern 3: Context Menus
**What:** Right-click menus on entries
**When to use:** For delete, duplicate, move actions
**Example:**
```rust
let response = ui.label(&entry.content);
response.context_menu(|ui| {
    if ui.button("Delete").clicked() {
        // Handle delete
        ui.close_menu();
    }
    if ui.button("Duplicate").clicked() {
        // Handle duplicate
        ui.close_menu();
    }
});
```

### Pattern 4: Date Navigation
**What:** Previous/next day navigation
**When to use:** Arrow buttons, keyboard shortcuts
**Example:**
```rust
use chrono::Days;

// Previous day
state.current_date = state.current_date.checked_sub_days(Days::new(1))
    .unwrap_or(state.current_date);

// Next day
state.current_date = state.current_date.checked_add_days(Days::new(1))
    .unwrap_or(state.current_date);
```

### Anti-Patterns to Avoid
- **Storing state in widgets:** egui is immediate mode - widgets don't retain state
- **Deep nesting of ui.horizontal/vertical:** Makes layout brittle, use ui.scope()
- **Blocking operations in update():** Use background threads for DB access
- **Not consuming keyboard events:** Use consume_key() not key_pressed() for shortcuts

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown rendering | Custom parser | egui_commonmark | Complex, handles edge cases |
| Date arithmetic | Manual day calc | chrono checked_add_days | Handles month/year boundaries |
| Time snapping | Manual rounding | chrono with Timelike | Round to nearest 15 min |
| Text editing | Custom widget | TextEdit | Focus, selection, undo built-in |

**Key insight:** egui provides building blocks; compose them rather than replacing them.

## Common Pitfalls

### Pitfall 1: Focus Management in Immediate Mode
**What goes wrong:** TextEdit doesn't get focus when entering edit mode
**Why it happens:** In immediate mode, focus must be explicitly requested
**How to avoid:** Use `response.request_focus()` immediately after creating TextEdit
**Warning signs:** User has to click twice to start editing

### Pitfall 2: State Synchronization
**What goes wrong:** Edit buffer gets out of sync with DB
**Why it happens:** Not copying content to buffer when entering edit mode
**How to avoid:** Initialize edit_buffer when setting editing_entry_id
**Warning signs:** Old content appears when editing different entries

### Pitfall 3: Keyboard Shortcut Conflicts
**What goes wrong:** Shortcuts trigger in text fields
**Why it happens:** Not checking if a text widget has focus
**How to avoid:** Check `!ui.ctx().wants_keyboard_input()` before handling shortcuts
**Warning signs:** Ctrl+N creates entry while typing

### Pitfall 4: Click Detection Ambiguity
**What goes wrong:** Click-to-edit conflicts with link clicks
**Why it happens:** Markdown links also handle clicks
**How to avoid:** Use double-click for edit mode, single-click for links
**Warning signs:** Clicking links enters edit mode

### Pitfall 5: Time Snapping Edge Cases
**What goes wrong:** Entries snap to wrong time at day boundaries
**Why it happens:** Not handling 23:45 + 15 min correctly
**How to avoid:** Use chrono's checked arithmetic, clamp to day boundaries
**Warning signs:** Entries appearing on next day unexpectedly

## Code Examples

### Timeline Hour Rendering
```rust
// Source: egui patterns
fn render_timeline(ui: &mut Ui, state: &mut DiaryViewState, entries: &[DiaryEntry]) {
    let work_start = state.config.work_hours_start; // e.g., 9
    let work_end = state.config.work_hours_end;     // e.g., 17

    for hour in work_start..work_end {
        ui.horizontal(|ui| {
            ui.label(format!("{:02}:00", hour));
            ui.separator();
            // Render entries for this hour
            for entry in entries.iter().filter(|e| e.start_time.hour() == hour) {
                render_entry(ui, state, entry);
            }
        });
    }
}
```

### Markdown View Mode
```rust
// Source: egui_commonmark docs
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

fn render_entry_content(ui: &mut Ui, cache: &mut CommonMarkCache, content: &str) {
    CommonMarkViewer::new().show(ui, cache, content);
}
```

### Search Implementation
```rust
// Source: egui patterns
fn render_search(ui: &mut Ui, state: &mut DiaryViewState) {
    ui.horizontal(|ui| {
        ui.label("🔍");
        let response = ui.text_edit_singleline(&mut state.search_query);
        if response.changed() {
            if state.search_query.starts_with('#') {
                // Hashtag search
                state.search_results = Some(search_by_tag(&state.search_query[1..]));
            } else if !state.search_query.is_empty() {
                // Full-text search
                state.search_results = Some(search_entries(&state.search_query));
            } else {
                state.search_results = None;
            }
        }
    });
}
```

### Time Snapping Utility
```rust
// Source: chrono patterns
use chrono::{NaiveTime, Timelike};

fn snap_to_15_minutes(time: NaiveTime) -> NaiveTime {
    let minutes = time.minute();
    let snapped = ((minutes + 7) / 15) * 15; // Round to nearest 15
    let snapped = snapped.min(45); // Handle 60 -> 45 case
    NaiveTime::from_hms_opt(time.hour(), snapped, 0).unwrap()
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| egui retained mode | Immediate mode | N/A | State managed externally |
| egui_commonmark 0.18 | 0.21 for egui 0.32 | July 2025 | API simplification |
| Manual image loading | egui Image API | egui 0.23+ | Built-in image support |

**Deprecated/outdated:**
- `CommonMarkViewer::new()` used to take an ID - no longer required in 0.18+
- `syntax_theme` deprecated in favor of `syntax_theme_dark`/`syntax_theme_light`

## Open Questions

1. **Scroll position persistence**
   - What we know: ScrollArea maintains position per-frame
   - What's unclear: Best way to persist scroll position across sessions
   - Recommendation: Store scroll offset in app state, restore on load

2. **Multi-line entry height calculation**
   - What we know: egui TextEdit grows with content
   - What's unclear: How to limit height while allowing scroll within entry
   - Recommendation: Use `desired_rows()` for initial height, let it grow

## Sources

### Primary (HIGH confidence)
- egui docs.rs 0.32.0 - Response, TextEdit, Memory, InputState APIs
- egui_commonmark CHANGELOG.md - Version compatibility matrix
- chrono docs.rs - NaiveDate, NaiveTime, checked arithmetic

### Secondary (MEDIUM confidence)
- egui GitHub examples - Common patterns for immediate mode
- lampsitter/egui_commonmark README - Usage examples

### Tertiary (LOW confidence)
- Web searches for egui patterns - Community approaches

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Version compatibility verified from CHANGELOG
- Architecture: HIGH - Patterns from official egui documentation
- Pitfalls: MEDIUM - Derived from egui model + common immediate mode issues

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (30 days - stable libraries)

