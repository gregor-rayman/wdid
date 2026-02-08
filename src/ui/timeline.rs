use egui::{ScrollArea, Ui};
use egui_commonmark::CommonMarkCache;

use super::entry::{render_entry, EntryAction};
use super::state::DiaryViewState;
use crate::db::DiaryEntry;

/// Result of timeline rendering, containing any pending actions.
#[derive(Debug, Clone, Default)]
pub struct TimelineActions {
    /// Entry to save (id, content, start_time, duration)
    pub save: Option<(i64, String, String, Option<i32>)>,
    /// Entry to delete
    pub delete: Option<i64>,
}

/// Render the timeline view showing entries.
///
/// When `is_search_mode` is true, shows date prefixes for each entry.
/// Returns `TimelineActions` with any pending save/delete operations.
pub fn render_timeline(
    ui: &mut Ui,
    entries: &[DiaryEntry],
    state: &mut DiaryViewState,
    cache: &mut CommonMarkCache,
    is_search_mode: bool,
) -> TimelineActions {
    let mut actions = TimelineActions::default();

    if entries.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            if is_search_mode {
                ui.label(egui::RichText::new("No matching entries").weak());
                ui.add_space(10.0);
                ui.label("Try a different search term");
            } else {
                ui.label(egui::RichText::new("No entries for this date").weak());
                ui.add_space(10.0);
                ui.label("Press Ctrl+N to create a new entry");
            }
        });
        return actions;
    }

    // Show search result count
    if is_search_mode {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("🔍 {} result{}", entries.len(), if entries.len() == 1 { "" } else { "s" }))
                    .weak(),
            );
        });
        ui.add_space(8.0);
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut prev_time: Option<&str> = None;
            let mut prev_date: Option<&str> = None;

            for entry in entries {
                // In search mode, show date headers when date changes
                if is_search_mode {
                    if prev_date != Some(&entry.date) {
                        if prev_date.is_some() {
                            ui.add_space(12.0);
                            ui.separator();
                        }
                        ui.add_space(8.0);
                        // Format date nicely (parse YYYY-MM-DD)
                        let date_label = format_date_label(&entry.date);
                        ui.label(egui::RichText::new(date_label).strong());
                        prev_date = Some(&entry.date);
                        prev_time = None; // Reset time gap tracking for new date
                    }
                }

                // Add visual gap for entries 30+ minutes apart (same date)
                if !is_search_mode {
                    if let Some(prev) = prev_time {
                        if should_add_gap(prev, &entry.start_time) {
                            ui.add_space(16.0);
                            ui.separator();
                        }
                    }
                }

                ui.add_space(8.0);
                let action = render_entry(ui, state, entry, cache);

                // Collect any actions
                match action {
                    EntryAction::Save {
                        id,
                        content,
                        start_time,
                        duration,
                    } => {
                        actions.save = Some((id, content, start_time, duration));
                    }
                    EntryAction::Delete(id) => {
                        actions.delete = Some(id);
                    }
                    EntryAction::None => {}
                }

                prev_time = Some(&entry.start_time);
            }

            ui.add_space(16.0);
        });

    actions
}

/// Format a YYYY-MM-DD date string into a readable label.
fn format_date_label(date_str: &str) -> String {
    use chrono::NaiveDate;
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        date.format("%a, %b %d").to_string()
    } else {
        date_str.to_string()
    }
}

/// Check if there's a 30+ minute gap between two times (HH:MM format).
fn should_add_gap(prev: &str, current: &str) -> bool {
    let parse_minutes = |s: &str| -> Option<i32> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            let h: i32 = parts[0].parse().ok()?;
            let m: i32 = parts[1].parse().ok()?;
            Some(h * 60 + m)
        } else {
            None
        }
    };

    if let (Some(prev_m), Some(curr_m)) = (parse_minutes(prev), parse_minutes(current)) {
        curr_m - prev_m >= 30
    } else {
        false
    }
}

