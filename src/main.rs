mod app;
mod calendar;
mod config;
mod db;
mod error;
mod paths;
mod tray;
mod ui;

use app::WdidApp;
use eframe::egui;
use paths::AppPaths;

/// Embedded tray icon (32x32 PNG).
const TRAY_ICON: &[u8] = include_bytes!("../assets/icon.png");

fn main() -> eframe::Result<()> {
    // Set up paths
    let paths = AppPaths::new().expect("Could not determine app paths");
    paths.ensure_dirs().expect("Could not create app directories");

    // Spawn system tray (before creating window)
    let tray_rx = tray::spawn_tray(TRAY_ICON);

    // Load saved window state
    let window_state = crate::config::load_window_state(&paths.window_state_file);
    let width = window_state.width.unwrap_or(800.0);
    let height = window_state.height.unwrap_or(600.0);

    // Build viewport with saved size
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([width, height])
        .with_min_inner_size([400.0, 300.0]);

    // Only set position on X11 (WAYLAND_DISPLAY not set)
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        if let (Some(x), Some(y)) = (window_state.x, window_state.y) {
            viewport = viewport.with_position([x, y]);
        }
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "wdid",
        options,
        Box::new(move |cc| Ok(Box::new(WdidApp::new(cc, paths, tray_rx)))),
    )
}
