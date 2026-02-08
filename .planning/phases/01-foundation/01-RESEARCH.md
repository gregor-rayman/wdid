# Phase 1: Foundation & Data Layer - Research

**Researched:** 2026-02-08
**Domain:** SQLite persistence, XDG configuration, Rust desktop application
**Confidence:** HIGH

## Summary

Phase 1 establishes the data persistence and configuration infrastructure for wdid. The primary technologies are rusqlite for SQLite database access and the directories crate for XDG-compliant paths on Linux.

The critical insight is that rusqlite's `Connection` is `Send` but NOT `Sync`, meaning database access from the GUI thread requires either a dedicated database thread with channel-based communication, or careful single-threaded access. For Phase 1's simple use case (single-threaded startup, user-initiated saves), direct access is acceptable, but the architecture should anticipate future async needs.

**Primary recommendation:** Use rusqlite with bundled SQLite, WAL mode, and the directories crate for XDG paths. Keep database operations simple and synchronous initially; refactor to threaded access if needed later.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rusqlite | 0.38.0 | SQLite database access | De facto Rust SQLite wrapper, ergonomic API, bundled SQLite option |
| directories | 6.0.0 | XDG path resolution | Standard cross-platform paths, Linux XDG compliance |
| toml | 0.9.x | Config file parsing | Simple, serde integration, human-readable format |
| serde | 1.x | Serialization | Required for toml parsing, derive macros |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror | 2.x | Error types | Define custom error enums |
| anyhow | 1.x | Error handling | Application-level error propagation |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rusqlite | sqlx | sqlx is async, compile-time checked; rusqlite simpler for sync desktop |
| toml | serde_json | JSON less human-editable than TOML for config |

**Installation:**
```bash
cargo add rusqlite --features bundled
cargo add directories
cargo add toml
cargo add serde --features derive
cargo add thiserror
cargo add anyhow
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs           # eframe entry point
├── app.rs            # App struct, egui rendering
├── db/
│   ├── mod.rs        # Database module
│   ├── connection.rs # Connection management, WAL setup
│   └── schema.rs     # Table creation, migrations
├── config/
│   ├── mod.rs        # Config module
│   └── types.rs      # Config struct definitions
└── error.rs          # Custom error types
```

### Pattern 1: Database Initialization
**What:** Open database, enable WAL, create tables if needed
**When to use:** Application startup
**Example:**
```rust
// Source: https://docs.rs/rusqlite/0.38.0/rusqlite/
use rusqlite::{Connection, Result};

pub fn init_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(include_str!("schema.sql"))?;
    Ok(conn)
}
```

### Pattern 2: XDG Directory Setup
**What:** Get config/data directories, create if needed
**When to use:** First run, startup
**Example:**
```rust
// Source: https://docs.rs/directories/6.0.0/directories/
use directories::ProjectDirs;
use std::fs;

pub fn get_paths() -> Option<(PathBuf, PathBuf)> {
    let proj = ProjectDirs::from("", "wdid", "wdid")?;
    let config_dir = proj.config_dir().to_path_buf();  // ~/.config/wdid
    let data_dir = proj.data_local_dir().to_path_buf(); // ~/.local/share/wdid
    fs::create_dir_all(&config_dir).ok()?;
    fs::create_dir_all(&data_dir).ok()?;
    Some((config_dir, data_dir))
}
```

### Pattern 3: Config with Defaults
**What:** Parse TOML with graceful fallback for missing fields
**When to use:** Config loading
**Example:**
```rust
// Source: https://docs.rs/toml/0.9.11/toml/
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub calendars: Vec<CalendarFeed>,
}

pub fn load_config(path: &Path) -> Config {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}
```

### Anti-Patterns to Avoid
- **Blocking GUI thread with DB:** Never run queries on main thread if they might be slow
- **Hardcoded paths:** Always use directories crate, never hardcode ~/.config
- **Panicking on config errors:** Parse errors should show friendly message, not crash

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| XDG paths | Manual ~/.config detection | directories crate | Handles edge cases, cross-platform |
| TOML parsing | Custom parser | toml + serde | Proven, handles escaping, comments |
| SQLite binding | Raw FFI | rusqlite | Memory safety, ergonomic API |
| Prepared stmt cache | Manual HashMap | `prepare_cached()` | Built-in LRU, handles lifetimes |

**Key insight:** SQLite and config parsing have many edge cases. The standard crates handle them correctly.

## Common Pitfalls

### Pitfall 1: SQLite Connection Threading
**What goes wrong:** Sharing Connection across threads causes compile error or data races
**Why it happens:** Connection is Send (can move between threads) but NOT Sync (can't share)
**How to avoid:** Either:
  - Keep database on single thread (simple, Phase 1 approach)
  - Use dedicated thread + channels for DB operations
  - Use connection pool (r2d2-sqlite)
**Warning signs:** Compile errors about Sync bounds

### Pitfall 2: Prepared Statement Lifetimes
**What goes wrong:** Wrestling with lifetimes when storing prepared statements
**Why it happens:** Statement borrows Connection, complex lifetime management
**How to avoid:** Use `prepare_cached()` - rusqlite manages cache internally
**Warning signs:** Lifetime annotations spreading through codebase

### Pitfall 3: Config Migration
**What goes wrong:** App crashes when config has new/removed fields
**Why it happens:** Strict deserialization fails on unknown fields
**How to avoid:** Use `#[serde(default)]` on all fields, ignore unknown fields
**Warning signs:** Panic on app update with existing config

### Pitfall 4: Database Corruption Handling
**What goes wrong:** App crashes instead of recovering gracefully
**Why it happens:** SQLite errors propagated without handling
**How to avoid:** Wrap open in recovery logic: backup corrupted file, offer fresh start
**Warning signs:** Unhandled rusqlite::Error

### Pitfall 5: WAL Mode Files
**What goes wrong:** Users see extra -wal and -shm files, confused
**Why it happens:** WAL mode creates companion files
**How to avoid:** This is normal; document it or use DELETE mode (slower)
**Warning signs:** User reports about "extra files"

## Code Examples

### Complete Database Setup
```rust
// Source: rusqlite docs + SQLite WAL docs
use rusqlite::{Connection, Result};
use std::path::Path;

pub fn open_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;

    // Enable WAL for better concurrent access
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    // Create tables (idempotent)
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS diary_entries (
            id INTEGER PRIMARY KEY,
            date TEXT NOT NULL,
            start_time TEXT NOT NULL,
            duration INTEGER,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            event_uid TEXT,
            event_snapshot TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_entries_date ON diary_entries(date);
    "#)?;

    Ok(conn)
}
```

### Error-Tolerant Config Loading
```rust
// Source: toml + serde docs
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct Config {
    pub calendars: Vec<CalendarFeed>,
}

#[derive(Deserialize, Clone)]
pub struct CalendarFeed {
    pub name: String,
    pub url: String,
}

pub enum ConfigResult {
    Loaded(Config),
    Default(String), // reason
    Error(String),   // user-friendly message
}

pub fn load_config(path: &Path) -> ConfigResult {
    match std::fs::read_to_string(path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => ConfigResult::Loaded(config),
            Err(e) => ConfigResult::Error(format!(
                "Config file has errors: {}\nUsing defaults.", e
            )),
        },
        Err(_) => ConfigResult::Default("No config file found".into()),
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| diesel ORM | rusqlite direct | N/A | Simpler for desktop, less boilerplate |
| Manual XDG | directories crate | ~2020 | Standard practice now |
| Custom errors | thiserror/anyhow | ~2021 | Cleaner error handling |

**Note on versions:** The existing STACK.md suggests rusqlite 0.34.0, but current latest is 0.38.0. Use latest stable.

## Open Questions

1. **Database thread needed for Phase 1?**
   - What we know: Connection is Send but not Sync
   - What's unclear: Whether Phase 1 workload needs async DB access
   - Recommendation: Start synchronous, refactor if performance issues arise

2. **Config file hot-reloading?**
   - What we know: Not in Phase 1 scope
   - Recommendation: Defer to later phase if needed

## Sources

### Primary (HIGH confidence)
- rusqlite 0.38.0 docs (docs.rs) - Connection API, WAL setup
- directories 6.0.0 docs (docs.rs) - ProjectDirs API
- toml 0.9.x docs (docs.rs) - serde integration
- SQLite WAL documentation (sqlite.org) - WAL mode details

### Secondary (MEDIUM confidence)
- eframe_template GitHub - App structure patterns
- egui 0.32.x releases - Version verification

### Tertiary (LOW confidence)
- None - all findings verified with primary sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - docs.rs verified, current versions confirmed
- Architecture: HIGH - patterns from official documentation
- Pitfalls: HIGH - from PITFALLS.md research + documentation

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (30 days - stable ecosystem)

