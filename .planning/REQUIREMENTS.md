# Requirements: wdid

**Defined:** 2026-02-08
**Core Value:** See your day at a glance and capture what you did — calendar events and personal notes unified in a single timeline.

## v1 Requirements

### Diary Entries

- [ ] **DIARY-01**: User can create diary entry with start time
- [ ] **DIARY-02**: User can edit diary entry in place (raw markdown input, rendered display)
- [ ] **DIARY-03**: User can delete diary entry via right-click context menu
- [ ] **DIARY-04**: User can delete diary entry by clearing all text
- [ ] **DIARY-05**: Diary entries persist in SQLite database
- [ ] **DIARY-06**: User can link diary entry to a calendar event
- [ ] **DIARY-07**: User can set optional duration on diary entry
- [ ] **DIARY-08**: User can add hashtags to diary entries for categorization

### Calendar Integration

- [ ] **CAL-01**: App imports events from iCal feeds (URLs or .ics files)
- [ ] **CAL-02**: Calendar events display in left column of timeline
- [ ] **CAL-03**: User can manually refresh calendar feeds
- [ ] **CAL-04**: App automatically refreshes calendar feeds hourly
- [ ] **CAL-05**: App supports 3-5 calendar feeds configured via TOML
- [ ] **CAL-06**: Two-column layout shows events (left) and diary entries (right) aligned by time

### Search & Navigation

- [ ] **NAV-01**: User can navigate between days using arrow buttons
- [ ] **NAV-02**: App displays single-day view (today by default)
- [ ] **SRCH-01**: User can search entries by hashtag across all dates
- [ ] **SRCH-02**: Dedicated search box shows filtered results list

### System Integration

- [ ] **SYS-01**: App remembers window size and position between sessions
- [ ] **SYS-02**: Data stored locally in SQLite database
- [ ] **SYS-03**: System tray icon with left-click to show/hide window
- [ ] **SYS-04**: System tray right-click opens menu with options
- [ ] **SYS-05**: Closing window minimizes to system tray
- [ ] **SYS-06**: Quit option available in tray menu
- [ ] **SYS-07**: Config file at XDG location (~/.config/wdid/config.toml)

### Export

- [ ] **EXP-01**: User can export entries to Markdown format
- [ ] **EXP-02**: User can export entries to JSON format
- [ ] **EXP-03**: User can generate standup summary (what did I do today)
- [ ] **EXP-04**: User can generate weekly retro summary

## v2 Requirements

### Enhanced Navigation

- **NAV-03**: Calendar picker for jumping to specific dates
- **NAV-04**: Week view option
- **NAV-05**: Month view option

### Cross-Platform

- **PLAT-01**: Windows support
- **PLAT-02**: macOS support

### Advanced Features

- **ADV-01**: Keyboard shortcuts for common actions
- **ADV-02**: Dark/light theme toggle
- **ADV-03**: Entry templates

## Out of Scope

| Feature | Reason |
|---------|--------|
| Write-back to calendars | iCal feeds are read-only by design |
| Formal billing/invoicing | Time tracking is for personal reference only |
| Rich text editor (WYSIWYG) | Markdown is sufficient, keeps complexity low |
| Mobile or web versions | Desktop Linux is the focus |
| Calendar event creation | This is a viewer/diary, not a calendar manager |
| Cloud sync | Local-only storage, user controls their data |
| AI features | Adds complexity without serving core use case |
| Mood tracking / photos | Personal journaling features are out of scope |

## Traceability

<!-- Updated during roadmap creation -->

| Requirement | Phase | Status |
|-------------|-------|--------|
| — | — | — |

**Coverage:**
- v1 requirements: 28 total
- Mapped to phases: 0
- Unmapped: 28 ⚠️

---
*Requirements defined: 2026-02-08*
*Last updated: 2026-02-08 after initial definition*

