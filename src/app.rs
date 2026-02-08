use eframe::egui;
use egui_commonmark::CommonMarkCache;

use crate::config::{Config, ConfigResult};
use crate::db::Database;
use crate::paths::AppPaths;
use crate::ui::DiaryViewState;

#[allow(dead_code)]
pub struct WdidApp {
    db: Database,
    config: Config,
    config_warning: Option<String>,
    first_run: bool,
    view_state: DiaryViewState,
    markdown_cache: CommonMarkCache,
}

impl WdidApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Set up paths
        let paths = AppPaths::new().expect("Could not determine app paths");
        paths.ensure_dirs().expect("Could not create app directories");

        // Load config
        let (config, config_warning, first_run) =
            match crate::config::load_config(&paths.config_file) {
                ConfigResult::Loaded(c) => (c, None, false),
                ConfigResult::Created(c) => (c, None, true),
                ConfigResult::ParseError(msg) => (Config::default(), Some(msg), false),
            };

        // Open database
        let db = Database::open(&paths.database_file).expect("Could not open database");

        Self {
            db,
            config,
            config_warning,
            first_run,
            view_state: DiaryViewState::default(),
            markdown_cache: CommonMarkCache::default(),
        }
    }
}

impl eframe::App for WdidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show config warning if present
            if let Some(warning) = &self.config_warning {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠ {}", warning));
                ui.separator();
            }

            // Header with date navigation and search
            crate::ui::header::render_header(ui, &mut self.view_state);
            ui.separator();

            // Show welcome message on first run
            if self.first_run {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Welcome to wdid!");
                    ui.add_space(10.0);
                    ui.label("Start by adding a diary entry, or configure");
                    ui.label("calendar feeds in ~/.config/wdid/config.toml");
                });
            } else {
                ui.label("Ready for diary entries");
            }
        });
    }
}

