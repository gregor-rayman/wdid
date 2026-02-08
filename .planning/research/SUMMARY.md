# Project Research Summary

**Project:** wdid (What Did I Do)
**Domain:** Native Linux desktop app for daily work tracking
**Researched:** 2026-02-08
**Confidence:** MEDIUM-HIGH

## Executive Summary

wdid is a native Linux desktop application combining read-only calendar display with editable diary entries in a two-column timeline. This is a well-understood domain—personal productivity tools—but the Rust desktop ecosystem requires careful technology choices. Research indicates **egui 0.32.0** is the optimal GUI framework due to its accessibility support (AccessKit), active development, and excellent markdown ecosystem. The immediate mode paradigm simplifies state management for timeline views.

The recommended approach is to build foundation-first: establish database access patterns and async infrastructure before the GUI layer. Calendar integration (the differentiating feature) should come after core data layer is solid. System tray integration is fragmented on Linux (especially GNOME), so treat it as an enhancement rather than core functionality.

Key risks center on **threading discipline**—blocking the GUI thread is easy and catastrophic, and async runtime coexistence (tokio + GUI event loop) requires intentional design. The calcard library for iCal parsing is comprehensive but less battle-tested than alternatives; timezone handling and RRULE recurrence expansion need thorough testing.

## Key Findings

### Recommended Stack

The stack leverages pure-Rust crates to minimize system dependencies. egui provides accessibility and markdown support; rusqlite with bundled SQLite avoids version mismatches; reqwest with rustls eliminates OpenSSL dependency.

**Core technologies:**
- **egui 0.32.0 + eframe**: GUI framework — best accessibility, immediate mode, excellent markdown widgets
- **rusqlite 0.34.0**: SQLite database — use `bundled` feature, WAL mode for concurrency
- **tokio 1.x**: Async runtime — background calendar sync, channel communication with GUI
- **calcard**: iCal parsing — comprehensive RFC 5545 support from Stalwart Labs
- **tray-icon 0.21.x**: System tray — cross-platform (Linux/GTK, Windows, macOS)
- **egui_commonmark 0.22**: Markdown rendering — native egui integration

### Expected Features

**Must have (table stakes):**
- Day navigation with single-day view
- Diary entry creation with timestamps and markdown
- SQLite persistence with auto-save
- Basic hashtag search
- System tray with show/hide
- Basic export (markdown/plain text)

**Should have (differentiators):**
- Calendar integration (iCal feeds) — THIS IS THE CORE DIFFERENTIATOR
- Two-column timeline layout
- Linking diary entries to calendar events
- Standup/weekly summary export

**Defer (v2+):**
- Multiple iCal feed support
- Dark mode / theming
- PDF export
- Cloud sync (anti-feature: complexity explosion, privacy concerns)

### Architecture Approach

TEA (The Elm Architecture) with screen composition provides predictable state management. Unidirectional data flow (State → View → Message → Update → State) eliminates shared mutable state. Background work uses channels to communicate results back to the GUI thread.

**Major components:**
1. **App + Screens**: Root state with diary/settings screens owning their state
2. **Data Layer**: SQLite connection on dedicated thread, message-passing for queries
3. **Sync Service**: Background tokio runtime for iCal HTTP fetch/parse
4. **Tray Service**: Optional system tray with graceful degradation

### Critical Pitfalls

1. **Blocking GUI thread** — Use background threads for all I/O; never call blocking operations in update() or view()
2. **Async runtime conflicts** — Separate tokio runtime from GUI event loop; communicate via channels
3. **SQLite threading** — Connection is Send but not Sync; use dedicated thread or r2d2 pooling
4. **System tray fragmentation** — GNOME requires AppIndicator extension; provide fallback UI
5. **iCal parsing edge cases** — Timezone handling and RRULE expansion are complex; test with real feeds

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation & Data Layer
**Rationale:** Other components depend on database and config infrastructure. Establish threading patterns early to avoid blocking GUI.
**Delivers:** Working data layer with SQLite, config management, XDG-compliant paths
**Addresses:** Data persistence, configuration persistence
**Avoids:** SQLite threading mistakes, XDG directory compliance issues, config migration problems

### Phase 2: Core GUI & Diary
**Rationale:** Depends on data layer. Build the essential diary functionality before adding calendar complexity.
**Delivers:** Day view with diary entry creation/editing, day navigation, basic search
**Uses:** egui + eframe, egui_commonmark for markdown
**Implements:** TEA pattern, screen composition

### Phase 3: Calendar Integration
**Rationale:** This is the differentiator but depends on stable GUI. iCal parsing is complex—isolate risks.
**Delivers:** iCal feed fetching, calendar event display, entry-to-event linking
**Uses:** calcard, reqwest, tokio runtime
**Avoids:** Tokio/GUI event loop conflicts, iCal parsing edge cases

### Phase 4: System Integration
**Rationale:** Polish features that enhance but aren't core to the value proposition.
**Delivers:** System tray, window position persistence, keyboard shortcuts
**Avoids:** System tray fragmentation (graceful degradation)

### Phase 5: Export & Polish
**Rationale:** After core functionality is validated, add professional output features.
**Delivers:** Standup summary export, markdown/text export, final polish

### Phase Ordering Rationale

- **Foundation first**: Database and config are dependencies for everything else
- **GUI before calendar**: Validate core UX before adding sync complexity
- **Calendar before tray**: Calendar is the differentiator; tray is optional enhancement
- **Export last**: Requires stable data model and proven usage patterns

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3 (Calendar):** iCal parsing edge cases, timezone handling, RRULE expansion need testing with real feeds
- **Phase 4 (System Tray):** Linux DE fragmentation may need runtime detection

Phases with standard patterns (skip research-phase):
- **Phase 1 (Foundation):** Well-documented rusqlite patterns, standard XDG crate
- **Phase 2 (Core GUI):** egui has excellent documentation and examples

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Verified versions via web search, boringcactus 2025 survey |
| Features | MEDIUM | Based on multiple sources; specific user needs unvalidated |
| Architecture | HIGH | TEA pattern well-documented for egui; Halloy as reference |
| Pitfalls | MEDIUM-HIGH | Threading issues well-documented; some Linux-specific DE issues anecdotal |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **calcard library**: Less widely used than icalendar crate; may need fallback plan
- **egui + tray-icon coexistence**: Event loop integration less documented than GTK
- **Wayland window position**: Cannot read position (by design); size-only persistence

## Sources

### Primary (HIGH confidence)
- boringcactus 2025 Rust GUI survey — GUI framework comparison
- gtk-rs.org main event loop guide — threading patterns
- Iced architecture book — TEA pattern reference
- rusqlite/crates.io — version verification
- Halloy source (GitHub) — egui real-world architecture

### Secondary (MEDIUM confidence)
- Stalwart calcard GitHub — iCal parsing capabilities
- Community discussions — Linux tray fragmentation
- XDG Base Directory Specification — path conventions

---
*Research completed: 2026-02-08*
*Ready for roadmap: yes*

