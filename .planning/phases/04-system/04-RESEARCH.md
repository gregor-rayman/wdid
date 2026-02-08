# Phase 4: System Integration - Research

**Researched:** 2026-02-08
**Domain:** Desktop Integration (System Tray, Window Persistence)
**Confidence:** MEDIUM

## Summary

Phase 4 requires integrating a Rust/egui application as a well-behaved desktop citizen with system tray functionality and window persistence. The key challenge is that **eframe/egui does not natively support minimize-to-tray** - platform-specific code is required.

For system tray, the `tray-icon` crate (from Tauri project) is the standard choice. On Linux, it requires gtk, libxdo, and libappindicator. The critical insight from GitHub Discussion #737 is that window show/hide must be done via raw window handles and platform APIs because `ViewportCommand::Visible` doesn't work when the window is already hidden.

**Primary recommendation:** Use `tray-icon` crate with platform-specific window control via `raw-window-handle` and `x11` or GTK APIs.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tray-icon | 0.21.3 | System tray icon and menu | From Tauri project, actively maintained, cross-platform |
| muda | 0.17 | Context menus | Used by tray-icon for menus, from Tauri |
| raw-window-handle | 0.6 | Platform window access | Standard for getting native handles from eframe |

### Linux-Specific
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| gtk | 0.18 | GTK event loop for tray | Required for tray-icon on Linux |
| libappindicator | 0.9 | System tray protocol | Required for GNOME/AppIndicator support |
| x11 | ~0.21 | X11 window control | For show/hide window on X11 |

### System Dependencies (Linux)
```bash
# Arch Linux / Manjaro
pacman -S gtk3 xdotool libappindicator-gtk3  # or libayatana-appindicator

# Debian / Ubuntu
sudo apt install libgtk-3-dev libxdo-dev libappindicator3-dev  # or libayatana-appindicator3-dev
```

## Architecture Patterns

### Window State Management
```rust
// Track visibility state globally (tray handler runs in different context)
static WINDOW_VISIBLE: Mutex<bool> = Mutex::new(true);
```

### Linux GTK Thread Pattern
```rust
// Linux requires separate gtk thread for tray-icon
#[cfg(target_os = "linux")]
std::thread::spawn(|| {
    gtk::init().unwrap();
    let _tray_icon = create_tray_icon();
    gtk::main();
});
```

### Close-to-Tray Pattern
```rust
// In App::update()
if ctx.input(|i| i.viewport().close_requested()) {
    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
    // Hide window via platform-specific code
    hide_window(window_handle);
}
```

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| System tray | Custom GTK/X11 tray code | `tray-icon` crate | Complex protocols, platform differences |
| Tray menus | Manual menu construction | `muda` crate | Proper event handling, accessibility |
| Icon loading | Manual PNG parsing | `image` crate | Format support, error handling |

## Common Pitfalls

### Pitfall 1: ViewportCommand::Visible Doesn't Work When Hidden
**What goes wrong:** Setting `Visible(true)` when window is already hidden does nothing
**Why it happens:** egui doesn't update when window is not interactable/invisible
**How to avoid:** Use platform-specific APIs via raw_window_handle
**Ref:** GitHub Discussion #737, Issue #3655

### Pitfall 2: Linux Tray Requires GTK Thread
**What goes wrong:** Tray icon doesn't appear or crashes
**Why it happens:** tray-icon on Linux uses libappindicator which needs gtk event loop
**How to avoid:** Spawn separate thread, call `gtk::init()`, then `gtk::main()`

### Pitfall 3: GNOME Without AppIndicator Extension
**What goes wrong:** Tray icon invisible on GNOME
**Why it happens:** GNOME removed legacy tray support; requires AppIndicator extension
**How to avoid:** Document requirement; provide fallback (app stays in taskbar)

### Pitfall 4: Wayland Position Persistence
**What goes wrong:** Cannot restore window position on Wayland
**Why it happens:** Wayland by design doesn't let apps control their position
**How to avoid:** Only persist window SIZE on Linux; accept position limitation
**Ref:** .planning/STATE.md technical discovery

## Code Examples

### Tray Icon Setup with Menu
```rust
// Source: tray-icon docs + winit example
use tray_icon::{TrayIconBuilder, TrayIconEvent, menu::{Menu, MenuItem}};

let menu = Menu::new();
let show_item = MenuItem::with_id("show", "Show Window", true, None);
let quit_item = MenuItem::with_id("quit", "Quit", true, None);
menu.append(&show_item)?;
menu.append(&quit_item)?;

let icon = load_icon(include_bytes!("../assets/icon.png"))?;
let tray_icon = TrayIconBuilder::new()
    .with_menu(Box::new(menu))
    .with_tooltip("wdid - What Did I Do")
    .with_icon(icon)
    .build()?;
```

### Event Handling Pattern
```rust
// Source: GitHub Discussion #737
TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
    match event {
        TrayIconEvent::Click { button_state: MouseButtonState::Down, .. } => {
            toggle_window_visibility(handle);
        }
        _ => {}
    }
}));

MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
    match event.id.as_ref() {
        "show" => show_window(handle),
        "quit" => std::process::exit(0),
        _ => {}
    }
}));
```

### Icon Loading
```rust
// Source: tray-icon examples
fn load_icon(bytes: &[u8]) -> tray_icon::Result<tray_icon::Icon> {
    let image = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    tray_icon::Icon::from_rgba(image.into_raw(), width, height)
}
```

## Window Persistence

### eframe Storage API
eframe provides persistence for app state but NOT for window geometry:
```rust
// eframe::Storage trait - for app-level persistence only
fn get_value::<T: DeserializeOwned>(storage: &dyn Storage, key: &str) -> Option<T>
fn set_value<T: Serialize>(storage: &mut dyn Storage, key: &str, value: &T)
```

### Manual Window Persistence Required
Window size/position must be saved manually to config or dedicated file:
```rust
// Reading current window geometry (in update())
let info = ctx.input(|i| i.viewport());
let size = info.inner_rect.map(|r| r.size());  // Current window size
let pos = info.outer_rect.map(|r| r.min);      // Current position (X11 only)

// Setting initial geometry (in NativeOptions)
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([saved_width, saved_height])
        .with_position([saved_x, saved_y]),  // X11 only
    ..Default::default()
};
```

### Wayland Limitation
**CRITICAL:** On Wayland, window position cannot be read or set by applications.
Only persist window SIZE. Detect Wayland via `$XDG_SESSION_TYPE` or `$WAYLAND_DISPLAY`.

## ViewportCommand API Reference

Relevant commands from egui 0.33 (current: 0.32 in project):
| Command | Purpose |
|---------|---------|
| `Close` | Request viewport close |
| `CancelClose` | Cancel pending close (for close-to-tray) |
| `Visible(bool)` | Show/hide window (unreliable when hidden) |
| `Minimized(bool)` | Minimize/restore window |
| `Focus` | Bring window to foreground |
| `OuterPosition(Pos2)` | Set window position |
| `InnerSize(Vec2)` | Set window size |

### Detecting Close Request
```rust
// In App::update()
if ctx.input(|i| i.viewport().close_requested()) {
    // User clicked X button
    ctx.send_viewport_cmd(ViewportCommand::CancelClose);  // Prevent actual close
    // Then hide window
}
```

## Icon Requirements

| Property | Requirement |
|----------|-------------|
| Format | RGBA pixels (loaded via `image` crate) |
| Sizes | 16x16, 22x22, 24x24, or 32x32 typical for Linux |
| Transparency | Supported (alpha channel) |
| File format | PNG recommended |

## Open Questions

1. **X11 show/hide API choice:** Research found Windows-specific code but Linux equivalent
   needs either `x11` crate with `XMapWindow`/`XUnmapWindow` or GTK widget control.
   Recommendation: Use GTK approach since we already need GTK for tray.

2. **Wayland window show/hide:** Wayland compositor controls window visibility.
   May need `wl_surface_attach(NULL)` or layer-shell protocol.
   Recommendation: Start with X11 support; Wayland can remain as taskbar app.

## Sources

### Primary (HIGH confidence)
- [tray-icon docs.rs](https://docs.rs/tray-icon/0.21.3) - API, platform requirements
- [egui ViewportCommand](https://docs.rs/egui/0.33.3/egui/viewport/enum.ViewportCommand.html) - Full command list
- [tray-icon winit example](https://github.com/tauri-apps/tray-icon/blob/dev/examples/winit.rs) - Linux GTK pattern

### Secondary (MEDIUM confidence)
- [GitHub Discussion #737](https://github.com/emilk/egui/discussions/737) - Minimize-to-tray workaround
- [eframe docs.rs](https://docs.rs/eframe/0.33.3) - Storage, NativeOptions API

### Tertiary (LOW confidence)
- X11/Wayland window control specifics (needs validation during implementation)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - tray-icon is well-documented, from Tauri
- Architecture: MEDIUM - Windows pattern confirmed, Linux needs adaptation
- Pitfalls: HIGH - GitHub discussions document real issues
- Window persistence: MEDIUM - eframe API clear, platform limits documented

**Research date:** 2026-02-08
**Valid until:** 30 days (stable ecosystem)

