# Roadmap: wdid

**Created:** 2026-02-08
**Depth:** Standard (5 phases)
**Coverage:** 29/29 v1 requirements mapped ✓

## Overview

wdid is built in five phases: Foundation establishes data persistence and configuration, Core GUI delivers diary functionality, Calendar Integration adds the differentiating iCal feature, System Integration handles tray and window behavior, and Export & Polish completes the package with summary generation.

## Progress

| Phase | Name | Status | Progress |
|-------|------|--------|----------|
| 1 | Foundation & Data Layer | Not Started | ░░░░░ |
| 2 | Core GUI & Diary | Not Started | ░░░░░ |
| 3 | Calendar Integration | Not Started | ░░░░░ |
| 4 | System Integration | Not Started | ░░░░░ |
| 5 | Export & Polish | Not Started | ░░░░░ |

---

## Phase 1: Foundation & Data Layer

**Goal:** Establish reliable data persistence and configuration infrastructure that all features depend on.

**Dependencies:** None (foundational)

**Requirements:**
- DIARY-05: Diary entries persist in SQLite database
- SYS-02: Data stored locally in SQLite database
- SYS-07: Config file at XDG location (~/.config/wdid/config.toml)

**Success Criteria:**
1. User can start the app and it creates necessary directories/files automatically
2. Diary entry saved survives app restart
3. Config file at ~/.config/wdid/config.toml is created on first run
4. Invalid config displays helpful error message (doesn't crash)

---

## Phase 2: Core GUI & Diary

**Goal:** Users can create, edit, and organize diary entries in a navigable daily view.

**Dependencies:** Phase 1 (data layer)

**Requirements:**
- DIARY-01: User can create diary entry with start time
- DIARY-02: User can edit diary entry in place (raw markdown input, rendered display)
- DIARY-03: User can delete diary entry via right-click context menu
- DIARY-04: User can delete diary entry by clearing all text
- DIARY-07: User can set optional duration on diary entry
- DIARY-08: User can add hashtags to diary entries for categorization
- NAV-01: User can navigate between days using arrow buttons
- NAV-02: App displays single-day view (today by default)
- SRCH-01: User can search entries by hashtag across all dates
- SRCH-02: Dedicated search box shows filtered results list

**Success Criteria:**
1. User can create a new diary entry, see it in the timeline, and it persists after restart
2. User can edit an entry's text and see markdown rendered after saving
3. User navigates to yesterday, creates an entry, returns to today — both entries visible on their respective days
4. User types #project-x in an entry, uses search box to find all #project-x entries
5. User can delete entries via right-click or by clearing text

---

## Phase 3: Calendar Integration

**Goal:** Users can see their calendar events alongside diary entries in a unified timeline.

**Dependencies:** Phase 2 (GUI framework)

**Requirements:**
- CAL-01: App imports events from iCal feeds (URLs or .ics files)
- CAL-02: Calendar events display in left column of timeline
- CAL-03: User can manually refresh calendar feeds
- CAL-04: App automatically refreshes calendar feeds hourly
- CAL-05: App supports 3-5 calendar feeds configured via TOML
- CAL-06: Two-column layout shows events (left) and diary entries (right) aligned by time
- DIARY-06: User can link diary entry to a calendar event

**Success Criteria:**
1. User adds iCal URL to config, restarts app, sees calendar events in left column
2. User clicks refresh button and sees updated events (simulated by changing remote feed)
3. User sees events and diary entries side-by-side, aligned by time of day
4. User clicks "add note" on a calendar event and creates a linked diary entry
5. Multiple calendar feeds display together (distinguished by color or label)

---

## Phase 4: System Integration

**Goal:** App behaves as a well-integrated desktop application with tray, persistence, and expected behaviors.

**Dependencies:** Phase 2 (window exists)

**Requirements:**
- SYS-01: App remembers window size and position between sessions
- SYS-03: System tray icon with left-click to show/hide window
- SYS-04: System tray right-click opens menu with options
- SYS-05: Closing window minimizes to system tray
- SYS-06: Quit option available in tray menu

**Success Criteria:**
1. User resizes window, restarts app — window appears at same size and position
2. User clicks X to close — window hides, app continues in tray
3. User left-clicks tray icon — window toggles visibility
4. User right-clicks tray icon, selects Quit — app exits completely
5. App runs unobtrusively in background without consuming excessive resources

---

## Phase 5: Export & Polish

**Goal:** Users can extract summaries and data for use in standups, retrospectives, and backups.

**Dependencies:** Phase 2 (diary entries), Phase 3 (calendar events for context)

**Requirements:**
- EXP-01: User can export entries to Markdown format
- EXP-02: User can export entries to JSON format
- EXP-03: User can generate standup summary (what did I do today)
- EXP-04: User can generate weekly retro summary

**Success Criteria:**
1. User exports today's entries to Markdown file — file is human-readable and complete
2. User exports entries to JSON — valid JSON with all entry data
3. User generates standup summary — concise list of today's work suitable for team standup
4. User generates weekly summary — grouped by day, includes calendar context

---

## Coverage Map

| Requirement | Phase | Category |
|-------------|-------|----------|
| DIARY-01 | 2 | Diary |
| DIARY-02 | 2 | Diary |
| DIARY-03 | 2 | Diary |
| DIARY-04 | 2 | Diary |
| DIARY-05 | 1 | Diary |
| DIARY-06 | 3 | Diary |
| DIARY-07 | 2 | Diary |
| DIARY-08 | 2 | Diary |
| CAL-01 | 3 | Calendar |
| CAL-02 | 3 | Calendar |
| CAL-03 | 3 | Calendar |
| CAL-04 | 3 | Calendar |
| CAL-05 | 3 | Calendar |
| CAL-06 | 3 | Calendar |
| NAV-01 | 2 | Navigation |
| NAV-02 | 2 | Navigation |
| SRCH-01 | 2 | Search |
| SRCH-02 | 2 | Search |
| SYS-01 | 4 | System |
| SYS-02 | 1 | System |
| SYS-03 | 4 | System |
| SYS-04 | 4 | System |
| SYS-05 | 4 | System |
| SYS-06 | 4 | System |
| SYS-07 | 1 | System |
| EXP-01 | 5 | Export |
| EXP-02 | 5 | Export |
| EXP-03 | 5 | Export |
| EXP-04 | 5 | Export |

---
*Roadmap created: 2026-02-08*

