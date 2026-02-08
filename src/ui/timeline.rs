use egui::{ScrollArea, Ui};
use egui_commonmark::CommonMarkCache;

use crate::db::DiaryEntry;
use super::entry::render_entry_view;

/// Render the timeline view showing all entries for the current date.
pub fn render_timeline(
    ui: &mut Ui,
    entries: &[DiaryEntry],
    cache: &mut CommonMarkCache,
) {
    if entries.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(egui::RichText::new("No entries for this date").weak());
            ui.add_space(10.0);
            ui.label("Press Ctrl+N to create a new entry");
        });
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut prev_time: Option<&str> = None;

            for entry in entries {
                // Add visual gap for entries 30+ minutes apart
                if let Some(prev) = prev_time {
                    if should_add_gap(prev, &entry.start_time) {
                        ui.add_space(16.0);
                        ui.separator();
                    }
                }

                ui.add_space(8.0);
                render_entry_view(ui, cache, entry);

                prev_time = Some(&entry.start_time);
            }
            
            ui.add_space(16.0);
        });
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

