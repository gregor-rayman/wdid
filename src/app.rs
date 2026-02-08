use eframe::egui;
use egui::Key;
use egui_commonmark::CommonMarkCache;

use crate::config::{Config, ConfigResult};
use crate::db::{Database, DiaryEntry, NewDiaryEntry};
use crate::paths::AppPaths;
use crate::ui::{snap_to_15_minutes, DiaryViewState};

#[allow(dead_code)]
pub struct WdidApp {
    db: Database,
    config: Config,
    config_warning: Option<String>,
    first_run: bool,
    view_state: DiaryViewState,
    markdown_cache: CommonMarkCache,
    /// Entries for the currently displayed date
    entries: Vec<DiaryEntry>,
    /// Track which date entries were loaded for
    entries_date: Option<chrono::NaiveDate>,
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
            entries: Vec::new(),
            entries_date: None,
        }
    }

    /// Load entries for the given date from the database.
    fn load_entries(&mut self) {
        let date_str = self.view_state.current_date.format("%Y-%m-%d").to_string();
        match self.db.get_entries_for_date(&date_str) {
            Ok(entries) => {
                self.entries = entries;
                self.entries_date = Some(self.view_state.current_date);
            }
            Err(e) => {
                eprintln!("Failed to load entries: {}", e);
                self.entries = Vec::new();
            }
        }
    }

    /// Create a new entry at the current (snapped) time.
    fn create_entry(&mut self) {
        let now = chrono::Local::now().time();
        let snapped = snap_to_15_minutes(now);
        let time_str = snapped.format("%H:%M").to_string();
        let date_str = self.view_state.current_date.format("%Y-%m-%d").to_string();

        let new_entry = NewDiaryEntry {
            date: date_str,
            start_time: time_str,
            duration: None,
            content: String::new(),
            event_uid: None,
            event_snapshot: None,
        };

        match self.db.save_entry(&new_entry) {
            Ok(_id) => {
                // Reload entries to show the new one
                self.load_entries();
            }
            Err(e) => {
                eprintln!("Failed to create entry: {}", e);
            }
        }
    }
}

impl eframe::App for WdidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Reload entries if date changed
        if self.entries_date != Some(self.view_state.current_date) {
            self.load_entries();
        }

        // Handle Ctrl+N for new entry (only if not typing in a text field)
        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                if i.key_pressed(Key::N) && i.modifiers.command {
                    // Note: modifiers.command is Ctrl on Linux/Windows, Cmd on Mac
                    return true;
                }
                false
            })
            .then(|| self.create_entry());
        }

        // Track pending actions from timeline
        let mut needs_reload = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            // Show config warning if present
            if let Some(warning) = &self.config_warning {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠ {}", warning));
                ui.separator();
            }

            // Header with date navigation and search
            crate::ui::header::render_header(ui, &mut self.view_state);
            ui.separator();

            // Show welcome message on first run (only when no entries exist)
            if self.first_run && self.entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Welcome to wdid!");
                    ui.add_space(10.0);
                    ui.label("Start by adding a diary entry (Ctrl+N), or configure");
                    ui.label("calendar feeds in ~/.config/wdid/config.toml");
                });
            } else {
                // Render the timeline
                let actions = crate::ui::timeline::render_timeline(
                    ui,
                    &self.entries,
                    &mut self.view_state,
                    &mut self.markdown_cache,
                );

                // Handle save action
                if let Some((id, content, start_time, duration)) = actions.save {
                    // Delete entry if content is empty
                    if content.trim().is_empty() {
                        if let Err(e) = self.db.delete_entry(id) {
                            eprintln!("Failed to delete empty entry: {}", e);
                        }
                        needs_reload = true;
                    } else {
                        if let Err(e) =
                            self.db
                                .update_entry_full(id, &content, &start_time, duration)
                        {
                            eprintln!("Failed to update entry: {}", e);
                        }
                        needs_reload = true;
                    }
                }

                // Handle delete action
                if let Some(id) = actions.delete {
                    if let Err(e) = self.db.delete_entry(id) {
                        eprintln!("Failed to delete entry: {}", e);
                    }
                    needs_reload = true;
                }
            }
        });

        // Reload entries if any changes were made
        if needs_reload {
            self.load_entries();
        }
    }
}

