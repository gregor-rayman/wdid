mod app;
mod config;
mod db;
mod error;
mod paths;
mod ui;

use app::WdidApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native("wdid", options, Box::new(|cc| Ok(Box::new(WdidApp::new(cc)))))
}
