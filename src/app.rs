use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use chrono::{DateTime, Duration, Local};
use eframe::egui;
use egui::Key;
use egui_commonmark::CommonMarkCache;

use crate::calendar::{parse_ical, spawn_calendar_worker, CalendarCommand, CalendarResult};
use crate::config::{Config, ConfigResult, WindowState};
use crate::db::{CachedFeed, Database, DiaryEntry, NewDiaryEntry};
use crate::export::{
    copy_to_clipboard, format_day_markdown, format_entries_json, format_standup,
    format_weekly_retro, save_to_file, ExportAction,
};
use crate::paths::AppPaths;
use crate::tray::TrayCommand;
use crate::ui::{snap_to_15_minutes, CalendarAction, DiaryViewState, HeaderAction};

/// Check if running on Wayland (WAYLAND_DISPLAY env var set)
fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}

/// Auto-refresh interval for calendar feeds (1 hour)
const AUTO_REFRESH_INTERVAL: Duration = Duration::hours(1);

/// Interval between window state saves (5 seconds)
const WINDOW_SAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

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
    /// Track which date calendar events were loaded for
    calendar_events_date: Option<chrono::NaiveDate>,
    /// Channel to send commands to the calendar worker
    calendar_tx: Sender<CalendarCommand>,
    /// Channel to receive results from the calendar worker
    calendar_rx: Receiver<CalendarResult>,
    /// Whether initial calendar refresh has been triggered
    calendar_refresh_triggered: bool,
    /// Last time we checked for auto-refresh
    last_refresh_check: DateTime<Local>,
    /// Path to window state file
    window_state_file: PathBuf,
    /// Last saved window state (for change detection)
    last_saved_window_state: WindowState,
    /// Last time window state was saved
    last_window_save: Instant,
    /// Channel to receive commands from system tray
    tray_rx: Receiver<TrayCommand>,
}

impl WdidApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        paths: AppPaths,
        tray_rx: Receiver<TrayCommand>,
    ) -> Self {
        // Load config
        let (config, config_warning, first_run) =
            match crate::config::load_config(&paths.config_file) {
                ConfigResult::Loaded(c) => (c, None, false),
                ConfigResult::Created(c) => (c, None, true),
                ConfigResult::ParseError(msg) => (Config::default(), Some(msg), false),
            };

        // Open database
        let db = Database::open(&paths.database_file).expect("Could not open database");

        // Spawn calendar worker
        let (calendar_tx, calendar_rx) = spawn_calendar_worker();

        Self {
            db,
            config,
            config_warning,
            first_run,
            view_state: DiaryViewState::default(),
            markdown_cache: CommonMarkCache::default(),
            entries: Vec::new(),
            entries_date: None,
            calendar_events_date: None,
            calendar_tx,
            calendar_rx,
            calendar_refresh_triggered: false,
            last_refresh_check: Local::now(),
            window_state_file: paths.window_state_file,
            last_saved_window_state: WindowState::default(),
            last_window_save: Instant::now(),
            tray_rx,
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

    /// Load calendar events for the current date from the database.
    /// Separates all-day events from timed events.
    fn load_calendar_events(&mut self) {
        let date = self.view_state.current_date;
        match self.db.get_calendar_events_for_date(date) {
            Ok(events) => {
                // Separate all-day from timed events
                let (all_day, timed): (Vec<_>, Vec<_>) =
                    events.into_iter().partition(|e| e.all_day);
                self.view_state.all_day_events = all_day;
                self.view_state.calendar_events = timed;
                self.calendar_events_date = Some(date);
            }
            Err(e) => {
                eprintln!("Failed to load calendar events: {}", e);
                self.view_state.all_day_events = Vec::new();
                self.view_state.calendar_events = Vec::new();
            }
        }
    }

    /// Process fetched iCal data: parse, cache, and reload events.
    fn process_feed_data(
        &mut self,
        feed_url: &str,
        data: &str,
        feed_name: Option<String>,
        feed_color: Option<String>,
    ) {
        // Calculate date range: current date +/- 31 days
        let today = Local::now().date_naive();
        let range_start = today - Duration::days(31);
        let range_end = today + Duration::days(31);

        // Find user_email for this feed from config
        let user_email = self
            .config
            .calendars
            .iter()
            .find(|f| f.url == feed_url)
            .and_then(|f| f.user_email.as_deref());

        // Parse the iCal data
        match parse_ical(
            data,
            feed_url,
            range_start,
            range_end,
            feed_name.clone(),
            feed_color.clone(),
            user_email,
        ) {
            Ok(events) => {
                eprintln!(
                    "Parsed {} events from {} (range: {} to {})",
                    events.len(),
                    feed_url,
                    range_start,
                    range_end
                );

                // Clear old events for this feed
                if let Err(e) = self.db.clear_feed_events(feed_url) {
                    eprintln!("Failed to clear old events for {}: {}", feed_url, e);
                }

                // Save new events to database
                for event in &events {
                    if let Err(e) = self.db.save_calendar_event(event) {
                        eprintln!("Failed to save event '{}': {}", event.summary, e);
                    }
                }

                // Update feed metadata (success)
                let now = Local::now();
                let feed = CachedFeed {
                    url: feed_url.to_string(),
                    name: feed_name,
                    color: feed_color,
                    last_refresh: Some(now.format("%Y-%m-%dT%H:%M:%S").to_string()),
                    last_error: None,
                };
                if let Err(e) = self.db.save_feed(&feed) {
                    eprintln!("Failed to save feed metadata: {}", e);
                }

                // Clear any previous error for this feed
                self.view_state.feed_errors.remove(feed_url);
                // Update last refresh time
                self.view_state
                    .feed_last_refresh
                    .insert(feed_url.to_string(), now);

                // Reload events for display
                self.load_calendar_events();
            }
            Err(e) => {
                eprintln!("Failed to parse iCal from {}: {}", feed_url, e);
                // Track as feed error
                self.view_state
                    .feed_errors
                    .insert(feed_url.to_string(), e.to_string());
            }
        }
    }

    /// Handle feed error: track error, but still load cached data.
    fn handle_feed_error(&mut self, feed_url: &str, error: &str) {
        eprintln!("Feed error {}: {}", feed_url, error);

        // Track the error for UI display
        self.view_state
            .feed_errors
            .insert(feed_url.to_string(), error.to_string());

        // Update feed metadata with error
        let feed = CachedFeed {
            url: feed_url.to_string(),
            name: None, // Keep existing name if any
            color: None,
            last_refresh: None, // Don't update refresh time on error
            last_error: Some(error.to_string()),
        };
        if let Err(e) = self.db.save_feed(&feed) {
            eprintln!("Failed to save feed error: {}", e);
        }

        // Still load cached events (stale data is better than no data)
        self.load_calendar_events();
    }

    /// Trigger a manual calendar refresh (if not already refreshing).
    fn trigger_calendar_refresh(&mut self) {
        if !self.view_state.calendar_refreshing && !self.config.calendars.is_empty() {
            self.view_state.calendar_refreshing = true;
            let _ = self
                .calendar_tx
                .send(CalendarCommand::RefreshAll(self.config.calendars.clone()));
        }
    }

    /// Save window state if it has changed since last save.
    fn save_window_state_if_changed(&mut self, ctx: &egui::Context) {
        let viewport_info = ctx.input(|i| i.viewport().clone());

        // Get current window dimensions
        // inner_rect is None on Wayland, use screen_rect as fallback
        let (width, height) = if let Some(inner_rect) = viewport_info.inner_rect {
            (inner_rect.width(), inner_rect.height())
        } else {
            let screen = ctx.screen_rect();
            (screen.width(), screen.height())
        };

        // Only save position on X11 (WAYLAND_DISPLAY not set)
        let (x, y) = if std::env::var("WAYLAND_DISPLAY").is_err() {
            viewport_info
                .outer_rect
                .map(|r| (Some(r.min.x), Some(r.min.y)))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };

        let current_state = WindowState {
            width: Some(width),
            height: Some(height),
            x,
            y,
        };

        // Check if state has changed
        let changed = current_state.width != self.last_saved_window_state.width
            || current_state.height != self.last_saved_window_state.height
            || current_state.x != self.last_saved_window_state.x
            || current_state.y != self.last_saved_window_state.y;

        if changed {
            if let Err(e) =
                crate::config::save_window_state(&self.window_state_file, &current_state)
            {
                eprintln!("Failed to save window state: {}", e);
            } else {
                self.last_saved_window_state = current_state;
            }
        }
    }

    /// Handle export actions triggered from the Export menu.
    fn handle_export_action(&self, action: ExportAction) {
        use chrono::{Datelike, Days};

        match action {
            ExportAction::None => {}
            ExportAction::DayMarkdownClipboard => {
                let md = format_day_markdown(&self.view_state.current_date, &self.entries);
                if let Err(e) = copy_to_clipboard(&md) {
                    eprintln!("Clipboard error: {}", e);
                }
            }
            ExportAction::DayMarkdownFile => {
                let md = format_day_markdown(&self.view_state.current_date, &self.entries);
                let name = format!("{}.md", self.view_state.current_date);
                save_to_file(&md, &name, "Markdown", "md");
            }
            ExportAction::DayJsonClipboard => {
                let json = format_entries_json(&self.entries);
                if let Err(e) = copy_to_clipboard(&json) {
                    eprintln!("Clipboard error: {}", e);
                }
            }
            ExportAction::DayJsonFile => {
                let json = format_entries_json(&self.entries);
                let name = format!("{}.json", self.view_state.current_date);
                save_to_file(&json, &name, "JSON", "json");
            }
            ExportAction::StandupClipboard => {
                let summary = format_standup(&self.entries);
                if let Err(e) = copy_to_clipboard(&summary) {
                    eprintln!("Clipboard error: {}", e);
                }
            }
            ExportAction::WeeklyRetroClipboard => {
                // Get Monday of current week
                let weekday = self.view_state.current_date.weekday().num_days_from_monday();
                let monday = self
                    .view_state
                    .current_date
                    .checked_sub_days(Days::new(weekday as u64))
                    .unwrap_or(self.view_state.current_date);
                let sunday = monday.checked_add_days(Days::new(6)).unwrap_or(monday);

                // Query date range
                let start = monday.format("%Y-%m-%d").to_string();
                let end = sunday.format("%Y-%m-%d").to_string();
                let week_entries = self.db.get_entries_for_date_range(&start, &end).unwrap_or_default();

                let retro = format_weekly_retro(&week_entries, &monday);
                if let Err(e) = copy_to_clipboard(&retro) {
                    eprintln!("Clipboard error: {}", e);
                }
            }
        }
    }
}

impl eframe::App for WdidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Store context for tray module to request repaints when window is hidden
        crate::tray::set_egui_context(ctx.clone());

        // Handle close-to-tray FIRST: hide window instead of quitting
        // Must be at the very start of update() to catch close_requested before it clears
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            // On Wayland, Visible(false) is a no-op, use Minimized instead
            if is_wayland() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
            crate::tray::set_visible(false);
            // Save window state before hiding
            self.save_window_state_if_changed(ctx);
            // Don't process rest of update when hiding to ensure viewport commands complete
            return;
        }

        // Trigger initial calendar refresh if configured feeds exist
        if !self.calendar_refresh_triggered && !self.config.calendars.is_empty() {
            self.calendar_refresh_triggered = true;
            self.last_refresh_check = Local::now();
            self.trigger_calendar_refresh();
        }

        // Check for hourly auto-refresh
        let now = Local::now();
        if now.signed_duration_since(self.last_refresh_check) >= AUTO_REFRESH_INTERVAL {
            self.last_refresh_check = now;
            self.trigger_calendar_refresh();
        }

        // Periodically save window state (every 5 seconds if changed)
        if self.last_window_save.elapsed() >= WINDOW_SAVE_INTERVAL {
            self.save_window_state_if_changed(ctx);
            self.last_window_save = Instant::now();
        }

        // Poll for tray commands (non-blocking)
        while let Ok(cmd) = self.tray_rx.try_recv() {
            match cmd {
                TrayCommand::Show => {
                    // On Wayland, Visible(true) is a no-op, use Minimized(false) instead
                    if is_wayland() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    } else {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    crate::tray::set_visible(true);
                }
                TrayCommand::Hide => {
                    // On Wayland, Visible(false) is a no-op, use Minimized(true) instead
                    if is_wayland() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    } else {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    }
                    crate::tray::set_visible(false);
                }
                TrayCommand::Quit => {
                    // Save window state before quitting
                    self.save_window_state_if_changed(ctx);
                    std::process::exit(0);
                }
            }
            ctx.request_repaint();
        }

        // Poll for calendar results (non-blocking)
        while let Ok(result) = self.calendar_rx.try_recv() {
            match result {
                CalendarResult::FeedData {
                    feed_url,
                    data,
                    feed_name,
                    feed_color,
                } => {
                    self.process_feed_data(&feed_url, &data, feed_name, feed_color);
                }
                CalendarResult::FeedError { feed_url, error } => {
                    self.handle_feed_error(&feed_url, &error);
                }
                CalendarResult::RefreshComplete => {
                    eprintln!("Calendar refresh complete");
                    self.view_state.calendar_refreshing = false;
                }
            }
            ctx.request_repaint();
        }

        // Reload entries if date changed
        if self.entries_date != Some(self.view_state.current_date) {
            self.load_entries();
        }

        // Reload calendar events if date changed
        if self.calendar_events_date != Some(self.view_state.current_date) {
            self.load_calendar_events();
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

            // Header with date navigation, search, and refresh button
            let has_calendars = !self.config.calendars.is_empty();
            let (header_action, export_action) =
                crate::ui::header::render_header(ui, &mut self.view_state, has_calendars);

            // Handle header actions
            if header_action == HeaderAction::RefreshCalendars {
                self.trigger_calendar_refresh();
            }

            // Handle export actions
            if export_action != ExportAction::None {
                self.handle_export_action(export_action);
            }

            // Handle search query changes
            if self.view_state.search_changed() {
                let query = self.view_state.search_query.trim();
                if query.is_empty() {
                    // Clear search results, return to day view
                    self.view_state.search_results = None;
                } else if query.starts_with('#') && query.len() > 1 {
                    // Hashtag search (strip the #)
                    match self.db.search_by_hashtag(&query[1..]) {
                        Ok(results) => self.view_state.search_results = Some(results),
                        Err(e) => eprintln!("Search error: {}", e),
                    }
                } else {
                    // Full-text search
                    match self.db.search_by_text(query) {
                        Ok(results) => self.view_state.search_results = Some(results),
                        Err(e) => eprintln!("Search error: {}", e),
                    }
                }
            }

            ui.separator();

            // Check if we're in search mode
            let is_search_mode = self.view_state.search_results.is_some();

            // Show welcome message on first run (only when no entries and not searching)
            if self.first_run && self.entries.is_empty() && !is_search_mode {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.heading("Welcome to wdid!");
                    ui.add_space(10.0);
                    ui.label("Start by adding a diary entry (Ctrl+N), or configure");
                    ui.label("calendar feeds in ~/.config/wdid/config.toml");
                });
            } else {
                // Take search results and calendar events temporarily to avoid borrow conflicts
                let search_results = self.view_state.search_results.take();
                let calendar_events = std::mem::take(&mut self.view_state.calendar_events);
                let all_day_events = std::mem::take(&mut self.view_state.all_day_events);

                let entries_to_display: &[DiaryEntry] = if let Some(ref results) = search_results {
                    results.as_slice()
                } else {
                    self.entries.as_slice()
                };

                // Render the timeline (or search results)
                let actions = crate::ui::timeline::render_timeline(
                    ui,
                    entries_to_display,
                    &calendar_events,
                    &all_day_events,
                    &mut self.view_state,
                    &mut self.markdown_cache,
                    is_search_mode,
                );

                // Restore borrowed data
                self.view_state.search_results = search_results;
                self.view_state.calendar_events = calendar_events;
                self.view_state.all_day_events = all_day_events;

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

                // Handle unlink action
                if let Some(id) = actions.unlink {
                    if let Err(e) = self.db.unlink_entry(id) {
                        eprintln!("Failed to unlink entry: {}", e);
                    }
                    needs_reload = true;
                }

                // Handle calendar action (add note to event)
                if let CalendarAction::AddNote {
                    event_uid,
                    summary,
                    start_time,
                    duration,
                    feed_color,
                } = actions.calendar_action
                {
                    // Build event snapshot: "color:summary" format
                    let snapshot = format!(
                        "{}:{}",
                        feed_color.as_deref().unwrap_or("#808080"),
                        summary
                    );

                    let date_str = self.view_state.current_date.format("%Y-%m-%d").to_string();
                    let new_entry = NewDiaryEntry {
                        date: date_str,
                        start_time,
                        duration,
                        content: String::new(),
                        event_uid: Some(event_uid),
                        event_snapshot: Some(snapshot),
                    };

                    match self.db.save_entry(&new_entry) {
                        Ok(_id) => {
                            needs_reload = true;
                        }
                        Err(e) => {
                            eprintln!("Failed to create linked entry: {}", e);
                        }
                    }
                }
            }
        });

        // Reload entries if any changes were made
        if needs_reload {
            self.load_entries();
        }
    }
}

