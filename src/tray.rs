//! System tray icon with menu and event handling.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Commands sent from tray to the main application.
#[derive(Debug, Clone)]
pub enum TrayCommand {
    /// Show the main window.
    Show,
    /// Hide the main window.
    Hide,
    /// Quit the application.
    Quit,
}

/// Tracks whether the window is currently visible.
/// Used by the tray click handler to toggle visibility.
static VISIBLE: AtomicBool = AtomicBool::new(true);

/// Set the visibility state (called from main app when visibility changes).
pub fn set_visible(visible: bool) {
    VISIBLE.store(visible, Ordering::SeqCst);
}

/// Spawn the system tray in a dedicated thread.
/// Returns a receiver for tray commands.
pub fn spawn_tray(icon_bytes: &[u8]) -> Receiver<TrayCommand> {
    let (tx, rx) = mpsc::channel();
    let icon_data = icon_bytes.to_vec();

    std::thread::spawn(move || {
        run_tray_loop(icon_data, tx);
    });

    rx
}

/// Run the tray event loop (called from dedicated thread).
fn run_tray_loop(icon_data: Vec<u8>, tx: Sender<TrayCommand>) {
    // Initialize GTK on Linux (required for tray-icon)
    #[cfg(target_os = "linux")]
    {
        if gtk::init().is_err() {
            eprintln!("Failed to initialize GTK for system tray");
            return;
        }
    }

    // Load the icon
    let icon = match load_icon(&icon_data) {
        Some(icon) => icon,
        None => {
            eprintln!("Failed to load tray icon");
            return;
        }
    };

    // Build the menu
    let menu = build_menu();

    // Create the tray icon
    let _tray: TrayIcon = match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("wdid - What Did I Do")
        .with_icon(icon)
        .build()
    {
        Ok(tray) => tray,
        Err(e) => {
            eprintln!("Failed to create tray icon: {}", e);
            return;
        }
    };

    // Set up event handlers
    setup_event_handlers(tx);

    // Run the event loop
    #[cfg(target_os = "linux")]
    gtk::main();

    // On non-Linux, just sleep (events are processed via channels)
    #[cfg(not(target_os = "linux"))]
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Load icon from PNG bytes.
fn load_icon(bytes: &[u8]) -> Option<Icon> {
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    Icon::from_rgba(rgba, width, height).ok()
}

/// Build the right-click context menu.
fn build_menu() -> Menu {
    let menu = Menu::new();

    let show_item = MenuItem::with_id("show", "Show", true, None);
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);

    let _ = menu.append(&show_item);
    let _ = menu.append(&quit_item);

    menu
}

/// Set up tray icon and menu event handlers.
fn setup_event_handlers(tx: Sender<TrayCommand>) {
    let tx_menu = tx.clone();

    // Handle menu events
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        match event.id.0.as_str() {
            "show" => {
                let _ = tx_menu.send(TrayCommand::Show);
                VISIBLE.store(true, Ordering::SeqCst);
            }
            "quit" => {
                let _ = tx_menu.send(TrayCommand::Quit);
            }
            _ => {}
        }
    }));

    // Handle tray icon click events (left-click to toggle)
    TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
        if let TrayIconEvent::Click { button, .. } = event {
            if button == tray_icon::MouseButton::Left {
                // Toggle visibility
                let currently_visible = VISIBLE.load(Ordering::SeqCst);
                if currently_visible {
                    let _ = tx.send(TrayCommand::Hide);
                    VISIBLE.store(false, Ordering::SeqCst);
                } else {
                    let _ = tx.send(TrayCommand::Show);
                    VISIBLE.store(true, Ordering::SeqCst);
                }
            }
        }
    }));
}

