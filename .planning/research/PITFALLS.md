# Pitfalls Research: Rust Desktop Applications

**Domain:** Native Linux desktop app with SQLite, iCal parsing, system tray
**Researched:** 2026-02-08
**Confidence:** MEDIUM-HIGH

## Critical Pitfalls

### Pitfall 1: Blocking the GUI Main Loop

**What goes wrong:**
Long-running operations (network requests, file I/O, database queries, iCal parsing) executed on the main thread freeze the entire UI. Users see unresponsive windows, grey overlays, or "Application Not Responding" dialogs.

**Why it happens:**
- Developers don't realize SQLite queries or iCal HTTP fetches can take seconds
- Rust's ownership makes moving state to background threads feel awkward
- GTK4/glib's event loop model differs from tokio/async-std expectations

**How to avoid:**
- Use `gio::spawn_blocking()` for synchronous blocking work (SQLite queries)
- Use `glib::spawn_future_local()` for async tasks on the glib main loop
- For tokio-dependent crates (like reqwest), spawn a separate tokio runtime and communicate via channels
- Never call `std::thread::sleep()` or blocking I/O on the main thread

**Warning signs:**
- UI freezes when fetching calendars or searching diary entries
- Window becomes unmovable during operations
- Compile warnings about `Send`/`Sync` when accessing GTK widgets from threads

**Phase to address:** Phase 1 (Core Infrastructure) — establish threading patterns early

**Sources:** [gtk-rs.org main event loop](https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html)

---

### Pitfall 2: GTK Objects Not Thread-Safe (NonNull<GObject> not Sync)

**What goes wrong:**
Attempting to access GTK widgets from background threads causes compile errors like:
`error[E0277]: NonNull<GObject> cannot be shared between threads safely`

**Why it happens:**
GTK objects are fundamentally single-threaded. Unlike Rust's Send+Sync model, GObject references cannot cross thread boundaries safely.

**How to avoid:**
- Use channels (async_channel, std::sync::mpsc, flume) to send *data* back to main thread
- Let the main thread update widgets based on received messages
- Use `glib::spawn_future_local()` with weak references: `clone!(#[weak] button, async move { ... })`
- Accept that background tasks return results; UI updates happen on main loop

**Warning signs:**
- Compiler errors mentioning `Sync` trait not implemented for GTK types
- Attempts to clone() widgets into thread closures
- Using `Arc<Mutex<Widget>>` patterns (wrong approach)

**Phase to address:** Phase 1 (Core Infrastructure)

**Sources:** [gtk-rs book](https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html), [2025 Rust GUI Survey](https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html)

---

### Pitfall 3: Tokio/Glib Event Loop Conflicts

**What goes wrong:**
Using tokio-dependent crates (reqwest, many HTTP clients) without proper runtime setup causes panics: "there is no reactor running, must be called from the context of a Tokio 1.x runtime"

**Why it happens:**
- Many async crates assume tokio runtime but don't document this
- GTK4 apps already run glib main loop; running `#[tokio::main]` blocks it
- Developers try to spawn tokio futures on glib's executor

**How to avoid:**
```rust
fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("tokio runtime"))
}
// Spawn tokio work:
runtime().spawn(async move { /* tokio-based work */ });
```
- Prefer async-std/smol compatible crates when possible (they work with glib)
- Use `ureq` (blocking) with `gio::spawn_blocking()` as simpler alternative to reqwest

**Warning signs:**
- Runtime panics mentioning "no reactor"
- App hangs at startup when using `#[tokio::main]`
- Deadlocks when awaiting tokio futures from glib context

**Phase to address:** Phase 2 (Calendar Integration) — when HTTP fetching begins

**Sources:** [gtk-rs tokio integration](https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html#tokio)

---

### Pitfall 4: Reference Cycles and Memory Leaks in GUI Callbacks

**What goes wrong:**
Closures capturing `Rc<RefCell<Widget>>` or similar patterns create reference cycles that never get freed, causing memory leaks that grow with app usage.

**Why it happens:**
- GUI patterns often want bidirectional references (parent↔child, button↔handler)
- Rust's `Rc`/`Arc` don't prevent cycles like a GC would
- GTK callbacks holding strong references to widgets that own them

**How to avoid:**
- Use GTK's `clone!` macro with `#[weak]` references for callbacks
- Prefer message-passing architectures over shared mutable state
- Use `Rc::downgrade()` / `Arc::downgrade()` for back-references
- Profile with valgrind/heaptrack periodically

**Warning signs:**
- Memory usage grows steadily while app is idle
- Widgets not being dropped when expected
- Complex `Rc<RefCell<Rc<RefCell<...>>>>` nesting in code

**Phase to address:** Phase 1 (Core Infrastructure)

---

### Pitfall 5: SQLite Connection Threading Mistakes

**What goes wrong:**
SQLite connection is Send but not Sync. Sharing one connection across threads via `Arc<Connection>` causes data races or "database locked" errors. Creating new connections per-query is inefficient.

**Why it happens:**
- Developers try to share rusqlite::Connection like a normal Rust value
- SQLite's threading modes are complex (serialized vs multi-thread)
- Confusion between connection pooling and connection sharing

**How to avoid:**
- Use `r2d2-sqlite` for connection pooling OR
- Use `tokio-rusqlite` for async access with internal worker thread
- For simple apps: keep single connection on dedicated thread, use channels for queries
- Always use WAL mode for better concurrent read/write

**Warning signs:**
- "database is locked" errors
- Compiler errors about Connection not being Sync
- Slow startup from creating connections repeatedly

**Phase to address:** Phase 1 (Core Infrastructure) — database setup

**Sources:** [rusqlite docs](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html), [r2d2-sqlite](https://github.com/ivanceras/r2d2-sqlite)

---

### Pitfall 6: Linux System Tray Fragmentation

**What goes wrong:**
System tray icons don't appear on some desktop environments, appear in wrong locations, or have missing functionality. GNOME has deprecated tray icons entirely.

**Why it happens:**
- Linux has competing tray protocols: legacy X11 XEmbed, StatusNotifierItem (SNI), AppIndicator
- GNOME removed tray support; requires extension ("AppIndicator and KStatusNotifierItem Support")
- Wayland compositors handle tray differently than X11
- Different DEs have different dbus service requirements

**How to avoid:**
- Use `tray-icon` crate (supports both XEmbed and StatusNotifierItem)
- Document that GNOME users need AppIndicator extension
- Test on multiple DEs: GNOME, KDE, XFCE minimum
- Provide fallback UI if tray unavailable (show/hide from dock)
- Consider tray as optional feature, not core functionality

**Warning signs:**
- Icon appears on KDE but not GNOME
- Tray works on X11 but not Wayland
- "Failed to register StatusNotifierItem" errors in logs

**Phase to address:** Phase 3 (System Tray & Window Management)

**Sources:** [tray-icon crate](https://lib.rs/crates/tray-icon), GNOME/KDE documentation

---

### Pitfall 7: iCal Parsing Edge Cases and RRULE Complexity

**What goes wrong:**
Calendar events don't show up, show at wrong times, or recurring events expand incorrectly. Timezone handling causes events to shift hours.

**Why it happens:**
- RFC 5545 (iCalendar) is complex; many feeds are non-compliant
- RRULE (recurrence rules) are particularly complex with EXDATE exceptions
- VTIMEZONE definitions vary between providers
- Google Calendar and Outlook produce different iCal flavors

**How to avoid:**
- Use `ical` crate for parsing + `rrule` crate for recurrence expansion
- Always normalize times to UTC internally, convert for display
- Test with multiple calendar sources: Google, Outlook, Apple, self-hosted
- Handle parse errors gracefully; don't crash on malformed feeds
- Cache parsed events; don't re-parse on every view

**Warning signs:**
- Events missing or duplicated
- All-day events showing on wrong day
- Recurring events not appearing or appearing infinitely

**Phase to address:** Phase 2 (Calendar Integration)

**Sources:** [rrule crate](https://github.com/fmeringdal/rust-rrule), RFC 5545

---

## Moderate Pitfalls

### Pitfall 8: Prepared Statement Lifetime Wrestling

**What goes wrong:**
Trying to store rusqlite::Statement alongside Connection in a struct causes lifetime errors. The borrow checker prevents caching prepared statements efficiently.

**Why it happens:**
- Statement borrows Connection; can't store sibling references in same struct
- Rust's lifetime system doesn't model this pattern well
- Developers try to cache statements for performance but hit ownership walls

**How to avoid:**
- Use `Connection::prepare_cached()` — rusqlite maintains internal LRU cache
- Don't try to store Statement in structs; prepare per-function
- Accept the pattern: connection.prepare_cached("SELECT ...").execute(...)
- For complex cases, use separate connection per statement cache

**Warning signs:**
- Lifetime errors involving `'a` and Connection
- Attempts to use `ouroboros` or self-referential struct crates
- Code with many explicit lifetime annotations around database code

**Phase to address:** Phase 1 (Core Infrastructure)

---

### Pitfall 9: Window State Persistence Race Conditions

**What goes wrong:**
Window position/size not saved correctly, saved too frequently (performance), or lost on crash. Saved coordinates wrong on multi-monitor setups.

**Why it happens:**
- Saving on every resize event is too frequent
- Not saving on graceful exit; only crash scenarios tested
- Monitor configuration changes make absolute coordinates meaningless
- Wayland doesn't allow reading window position (security model)

**How to avoid:**
- Debounce saves (e.g., 500ms after last resize)
- Save on application shutdown signal, not just window close
- On Wayland: can only save size, not position (by design)
- Store monitor identifier with position for multi-monitor setups
- Use TOML/JSON in XDG config directory

**Warning signs:**
- Window opens in wrong position after restart
- Config file written hundreds of times during resize
- Position "jumps" when opening on different monitor setup

**Phase to address:** Phase 3 (System Tray & Window Management)

---

### Pitfall 10: XDG Directory Compliance

**What goes wrong:**
App stores config/data in wrong locations, pollutes home directory with dotfiles, or fails to respect XDG environment variables.

**Why it happens:**
- Windows/macOS habits don't translate to Linux conventions
- Developers hardcode `~/.appname` instead of using XDG paths
- Not all XDG variables are set on all systems

**How to avoid:**
- Use `directories` crate for cross-platform path resolution
- Linux: `~/.config/wdid/`, `~/.local/share/wdid/`, `~/.cache/wdid/`
- Respect `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME` environment variables
- Separate config (TOML), data (SQLite), and cache (temporary files)

**Warning signs:**
- Files appearing directly in `~/.wdid` instead of `~/.config/wdid`
- Config paths hardcoded as strings
- App fails when XDG variables are explicitly set differently

**Phase to address:** Phase 1 (Core Infrastructure)

**Sources:** [directories crate](https://crates.io/crates/directories), XDG Base Directory Specification

---

### Pitfall 11: Config File Version Migration

**What goes wrong:**
App crashes or loses settings when config format changes between versions. Users get cryptic serde deserialization errors.

**Why it happens:**
- New fields added to config struct without defaults
- Removed fields cause serde to fail
- No version field to detect migration needs

**How to avoid:**
- Add `#[serde(default)]` to all optional/new fields
- Include config version field; migrate on load
- Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
- Write migration functions for breaking changes
- Backup old config before migrating

**Warning signs:**
- "missing field" errors after updates
- Settings reset unexpectedly
- Users reporting config file "corruption"

**Phase to address:** Phase 1 (Core Infrastructure)

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Single-threaded SQLite access | Simple code, no sync issues | UI freezes on large databases | MVP with small datasets |
| Hardcoded config paths | Quick development | Fails on non-standard systems | Never; use `directories` crate |
| `unwrap()` on iCal parsing | Skip error handling | Crashes on malformed feeds | Never; always handle parse errors |
| Clone widget data into closures | Avoid lifetime complexity | Memory overhead, potential leaks | Acceptable with `#[weak]` refs |
| Block on startup for DB init | Simpler init sequence | Slow cold starts | Acceptable for <100ms operations |
| In-memory calendar cache | Fast repeated access | Memory growth with many events | OK with bounded cache size |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| iCal HTTP fetch | Fetching on main thread | Use background task + channel |
| SQLite writes | Write-ahead without WAL mode | Enable WAL: `PRAGMA journal_mode=WAL` |
| System tray | Assuming all DEs support it | Graceful fallback, document GNOME extension |
| GTK + tokio | Running both event loops together | Separate runtime, channel communication |
| Window position (Wayland) | Expecting to read position | Accept Wayland limitation; save size only |
| Config file | Manual path construction | Use `directories` crate |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Parsing iCal on every scroll | UI lag when scrolling timeline | Parse once, cache structured data | >50 events |
| Unindexed SQLite queries | Slow hashtag search | Add indexes on frequently queried columns | >1000 entries |
| Full DB scan for date range | Slow date navigation | Index on date columns, query with bounds | >500 entries |
| Re-rendering entire timeline | Jank on any update | Virtual scrolling, partial redraws | >100 visible items |
| Synchronous HTTP timeout | UI freeze on slow network | Timeout + background thread | Any network request |
| No connection pooling | Connection overhead per query | Use r2d2 or reuse connection | >10 queries/second |

## "Looks Done But Isn't" Checklist

- [ ] **Calendar sync:** Often missing timezone handling — verify events show at correct local time
- [ ] **Diary save:** Often missing flush/sync — verify data persists after crash (not just clean exit)
- [ ] **Hashtag search:** Often missing case handling — verify case-insensitive search works
- [ ] **Window restore:** Often missing multi-monitor — verify works after monitor disconnect/reconnect
- [ ] **System tray:** Often missing GNOME — verify on GNOME with AppIndicator extension
- [ ] **Config load:** Often missing migration — verify old config files still load after update
- [ ] **Background sync:** Often missing error handling — verify network failures don't crash app
- [ ] **Markdown rendering:** Often missing escaping — verify special characters render correctly

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Blocked main loop | LOW | Identify blocking call, wrap in `spawn_blocking()` |
| Reference cycle leaks | MEDIUM | Audit closures, add `#[weak]` refs, profile memory |
| SQLite threading bugs | MEDIUM | Centralize DB access to single-thread model |
| Event loop conflicts | LOW | Separate tokio runtime, use channels |
| Wrong XDG paths | LOW | Use `directories` crate, migrate existing data |
| System tray missing | LOW | Add graceful degradation, document requirements |
| iCal parse failures | MEDIUM | Add error boundaries, test with real-world feeds |
| Config migration fails | HIGH | Add version field, write migrations, backup before change |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Blocking main loop | Phase 1 | UI stays responsive during DB queries |
| GTK not thread-safe | Phase 1 | Background tasks work without panics |
| Tokio/glib conflicts | Phase 2 | HTTP fetch works without runtime errors |
| Reference cycle leaks | Phase 1 | Memory stable over extended use |
| SQLite threading | Phase 1 | Concurrent reads work, no "locked" errors |
| System tray fragmentation | Phase 3 | Tray works on KDE, XFCE; graceful on GNOME |
| iCal parsing edge cases | Phase 2 | Events from Google/Outlook/Apple all display correctly |
| Prepared statement lifetimes | Phase 1 | Code compiles without lifetime hacks |
| Window state persistence | Phase 3 | Position/size restored after restart |
| XDG directory compliance | Phase 1 | Config/data in correct XDG directories |
| Config version migration | Phase 1 | Old configs load after updates |

## Rust-Specific Issues Summary

### Borrow Checker in GUI Context
- **Challenge:** GUI callbacks often need mutable access to shared state
- **Solution:** Message-passing (channels), `Rc<RefCell>` for single-threaded, gtk-rs's `clone!` macro with `#[weak]`
- **Anti-pattern:** `Arc<Mutex<Vec<Arc<Mutex<...>>>>>` nesting

### Async Runtime Coexistence
- **Challenge:** Multiple async runtimes (glib, tokio, async-std) have different expectations
- **Solution:** Pick one primary (glib for GTK apps), isolate others with dedicated threads
- **Anti-pattern:** `#[tokio::main]` in GTK app (blocks glib loop)

### Lifetime Ergonomics with FFI
- **Challenge:** GTK-rs objects have different lifetime semantics than pure Rust
- **Solution:** Follow gtk-rs patterns; use `clone!` macro, trust the bindings
- **Anti-pattern:** Fighting the borrow checker with `unsafe` or self-referential structs

---

## Sources

- [gtk-rs Main Event Loop Guide](https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html) — HIGH confidence
- [2025 Survey of Rust GUI Libraries](https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html) — HIGH confidence
- [rusqlite documentation](https://docs.rs/rusqlite/latest/rusqlite/) — HIGH confidence
- [directories crate](https://crates.io/crates/directories) — HIGH confidence
- [rust-rrule](https://github.com/fmeringdal/rust-rrule) — MEDIUM confidence
- Community discussions on Reddit r/rust — MEDIUM confidence
- XDG Base Directory Specification — HIGH confidence

---
*Pitfalls research for: Rust desktop application (wdid)*
*Researched: 2026-02-08*

