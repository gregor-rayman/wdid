# Architecture Patterns

**Domain:** Rust Desktop Application (GUI + Database + External Sync)
**Researched:** 2026-02-08

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         APPLICATION                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    GUI LAYER (Iced)                      │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐               │    │
│  │  │  Screen  │  │  Screen  │  │  Widget  │               │    │
│  │  │  (Diary) │  │(Settings)│  │(Timeline)│               │    │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘               │    │
│  │       │             │             │                      │    │
│  │       └─────────────┴─────────────┘                      │    │
│  │                     │                                    │    │
│  │              ┌──────▼──────┐                             │    │
│  │              │   Messages  │  ◄── User interactions      │    │
│  │              └──────┬──────┘      produce Messages       │    │
│  │                     │                                    │    │
│  │              ┌──────▼──────┐                             │    │
│  │              │    State    │  ◄── Single source of truth │    │
│  │              └──────┬──────┘                             │    │
│  └─────────────────────┼───────────────────────────────────┘    │
│                        │                                         │
│  ┌─────────────────────▼───────────────────────────────────┐    │
│  │                   ASYNC RUNTIME                          │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │    │
│  │  │    Tasks     │  │ Subscriptions│  │   Streams    │   │    │
│  │  │ (one-shot)   │  │  (polling)   │  │ (continuous) │   │    │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘   │    │
│  └─────────┼─────────────────┼─────────────────┼───────────┘    │
│            │                 │                 │                 │
│  ┌─────────▼─────────────────▼─────────────────▼───────────┐    │
│  │                   SERVICE LAYER                          │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐  │    │
│  │  │ Database │  │  Config  │  │   Sync   │  │  Tray   │  │    │
│  │  │ Service  │  │ Service  │  │ Service  │  │ Service │  │    │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬────┘  │    │
│  └───────┼─────────────┼─────────────┼─────────────┼───────┘    │
│          │             │             │             │             │
├──────────▼─────────────▼─────────────▼─────────────▼────────────┤
│                      EXTERNAL SYSTEMS                            │
│     ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐       │
│     │ SQLite  │   │  TOML   │   │  iCal   │   │ Desktop │       │
│     │   DB    │   │ Config  │   │ Feeds   │   │  (D-Bus)│       │
│     └─────────┘   └─────────┘   └─────────┘   └─────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| **App (main.rs)** | Bootstrap, window creation, root state | All screens, async runtime |
| **Screens** | Page-level UI and logic, own Message types | App via Actions, Widgets |
| **Widgets** | Reusable UI components | Screens via element mapping |
| **Database Service** | CRUD operations, queries | App via Task/Message |
| **Config Service** | Load/save TOML, XDG paths | App via Task/Message |
| **Sync Service** | Fetch iCal, parse events | App via Subscription |
| **Tray Service** | System tray icon, menu | App via platform events |

## Recommended Project Structure

```
src/
├── main.rs              # Entry point, Iced application bootstrap
├── app.rs               # Root App struct, state, update, view
├── message.rs           # Root Message enum
│
├── screen/              # Screen modules (page-level components)
│   ├── mod.rs           # Screen enum, routing
│   ├── diary.rs         # Main diary view (timeline + editor)
│   └── settings.rs      # Settings/preferences screen
│
├── widget/              # Reusable custom widgets
│   ├── mod.rs           # Re-exports
│   ├── timeline.rs      # Two-column timeline widget
│   ├── entry_editor.rs  # Markdown entry editor
│   └── day_nav.rs       # Day navigation controls
│
├── data/                # Data layer
│   ├── mod.rs           # Re-exports
│   ├── db.rs            # SQLite connection, migrations
│   ├── entry.rs         # DiaryEntry model + queries
│   └── event.rs         # CalendarEvent model + queries
│
├── sync/                # External data sync
│   ├── mod.rs           # Re-exports
│   └── ical.rs          # iCal fetching and parsing
│
├── config/              # Configuration
│   ├── mod.rs           # Re-exports
│   └── settings.rs      # Config struct, TOML serde, XDG paths
│
└── tray/                # System tray (optional)
    ├── mod.rs           # Re-exports
    └── linux.rs         # Linux tray implementation
```

## Architectural Patterns

### Pattern 1: The Elm Architecture (TEA)
**What:** Unidirectional data flow: State → View → Message → Update → State
**When:** Always — this is Iced's core pattern
**Why:** Predictable state management, easy debugging, no shared mutable state

```rust
struct App {
    current_day: NaiveDate,
    entries: Vec<DiaryEntry>,
    screen: Screen,
}

enum Message {
    DayChanged(NaiveDate),
    EntryLoaded(Vec<DiaryEntry>),
    Screen(screen::Message),
}

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::DayChanged(day) => {
            self.current_day = day;
            Task::perform(load_entries(day), Message::EntryLoaded)
        }
        // ...
    }
}
```

### Pattern 2: Screen Composition
**What:** Enum-based routing with each screen owning its state/messages
**When:** Multi-page applications
**Why:** Clean separation, each screen is self-contained

```rust
enum Screen {
    Diary(diary::State),
    Settings(settings::State),
}

enum Message {
    Diary(diary::Message),
    Settings(settings::Message),
}

// In view:
match &self.screen {
    Screen::Diary(state) => diary::view(state).map(Message::Diary),
    Screen::Settings(state) => settings::view(state).map(Message::Settings),
}
```

### Pattern 3: Action Pattern for Screen Communication
**What:** Screens return Actions instead of Tasks for parent coordination
**When:** Screen needs to trigger app-level changes (navigation, global state)
**Why:** Screens stay decoupled from app-level concerns

```rust
// In screen module:
pub enum Action {
    None,
    Run(Task<Message>),
    NavigateTo(Screen),
    SaveEntry(DiaryEntry),
}

pub fn update(state: &mut State, message: Message) -> Action {
    match message {
        Message::Save => Action::SaveEntry(state.current_entry.clone()),
        Message::GoSettings => Action::NavigateTo(Screen::Settings),
        _ => Action::None,
    }
}
```

### Pattern 4: Background Tasks via Subscription
**What:** Subscriptions for polling/background work
**When:** Periodic tasks (calendar refresh), external events
**Why:** Non-blocking, integrates cleanly with message system

```rust
fn subscription(&self) -> Subscription<Message> {
    iced::time::every(Duration::from_secs(3600))
        .map(|_| Message::RefreshCalendar)
}
```

### Pattern 5: Async Database Access
**What:** Database operations on background thread, results via messages
**When:** Any I/O operation (SQLite, file, network)
**Why:** Never block GUI thread

```rust
// Task spawns async work, sends message when complete
Task::perform(
    async move {
        let conn = rusqlite::Connection::open(&db_path)?;
        query_entries(&conn, date)
    },
    |result| Message::EntriesLoaded(result)
)
```


## Data Flow

### User Interaction Flow
```
User clicks "Next Day" button
         │
         ▼
    View produces Message::DayChanged(new_date)
         │
         ▼
    update() receives message
         │
         ├──► Updates state: self.current_day = new_date
         │
         └──► Returns Task::perform(load_entries(new_date), Message::EntriesLoaded)
                    │
                    ▼ (async, off GUI thread)
              Database query executes
                    │
                    ▼
              Message::EntriesLoaded(entries) dispatched
                    │
                    ▼
              update() receives, sets self.entries = entries
                    │
                    ▼
              view() re-renders with new entries
```

### Background Sync Flow
```
Subscription fires every hour
         │
         ▼
    Message::RefreshCalendar dispatched
         │
         ▼
    update() returns Task::perform(fetch_ical(urls), Message::CalendarFetched)
         │
         ▼ (async, off GUI thread)
    HTTP requests to iCal feeds
         │
         ▼
    Parse iCal data
         │
         ▼
    Message::CalendarFetched(events) dispatched
         │
         ▼
    update() saves to database, updates state
         │
         ▼
    view() shows updated calendar events
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Blocking the GUI Thread
**What:** Synchronous I/O in update() or view()
**Why bad:** UI freezes, poor user experience
**Instead:** Always use Task::perform for I/O, return immediately from update()

### Anti-Pattern 2: Shared Mutable State
**What:** Arc<Mutex<T>> for state shared between GUI and background threads
**Why bad:** Race conditions, complex reasoning, deadlocks
**Instead:** Message passing — background work sends messages back to GUI

### Anti-Pattern 3: Monolithic State
**What:** Single giant State struct with all app data
**Why bad:** Hard to reason about, ownership issues, redraws everything
**Instead:** Screen-level state with composition

### Anti-Pattern 4: Direct Database Calls in View
**What:** Querying database during view() function
**Why bad:** view() is called frequently, blocks rendering
**Instead:** Load data in update(), store in state, view() reads from state

### Anti-Pattern 5: Nested Message Matching Explosion
**What:** Deeply nested match arms in root update()
**Why bad:** Hard to maintain, violates single responsibility
**Instead:** Delegate to screen update() functions, use Action pattern

## Integration Points

| External System | Integration Approach | Crate |
|-----------------|---------------------|-------|
| **SQLite** | Background thread, message passing | `rusqlite` |
| **iCal Feeds** | HTTP fetch, parse | `reqwest`, `icalendar` |
| **TOML Config** | Load at startup, save on change | `serde`, `toml` |
| **XDG Directories** | Config/data paths | `directories` |
| **System Tray** | Platform-specific (Linux: D-Bus) | `ksni` or `tray-icon` |
| **Markdown** | Render in widgets | `pulldown-cmark` |

## Scaling Considerations

| Concern | At MVP | At Scale |
|---------|--------|----------|
| **Database size** | Single SQLite file | Same, SQLite handles millions of rows |
| **Calendar feeds** | Fetch sequentially | Fetch in parallel with `futures::join_all` |
| **Search** | Simple LIKE queries | SQLite FTS5 virtual table |
| **Widget complexity** | Basic layouts | Custom canvas widgets if needed |

## Build Order Implications

**Phase order based on dependencies:**

1. **Config + Data Layer First** — Other components need these
   - `config/` (settings struct, XDG paths)
   - `data/` (database, models)

2. **Core GUI Second** — Depends on data layer
   - `app.rs` (root state, update, view)
   - `screen/diary.rs` (main screen)
   - `widget/` (timeline, editor)

3. **Sync Layer Third** — Needs database to store results
   - `sync/ical.rs` (fetch, parse, store)

4. **Polish Last** — Optional enhancements
   - `tray/` (system tray)
   - `screen/settings.rs` (preferences)

## Rust-Specific Patterns

### Ownership in Iced
- **State owns data** — App struct owns all application data
- **Views borrow state** — `view(&self)` takes immutable reference
- **Messages are owned** — Messages move data to update()
- **Tasks capture by move** — Async closures own their captures

### Async Considerations
- **rusqlite is sync** — Wrap in `spawn_blocking` or dedicated thread
- **Iced runtime handles async** — Tasks execute on Iced's executor
- **No async in view()** — View is synchronous, reads from state only

## Sources

- Iced Architecture (HIGH): https://book.iced.rs/architecture.html
- Iced Documentation (HIGH): https://docs.rs/iced/latest/iced/
- Iced GitHub (HIGH): https://github.com/iced-rs/iced
- Halloy Source Structure (HIGH): https://github.com/squidowl/halloy/tree/main/src
- Iced Multi-page Pattern (MEDIUM): Community patterns from web search

