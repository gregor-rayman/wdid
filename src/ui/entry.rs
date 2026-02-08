use egui::Ui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::db::DiaryEntry;

/// Render a single diary entry with time badge and markdown content.
pub fn render_entry_view(ui: &mut Ui, cache: &mut CommonMarkCache, entry: &DiaryEntry) {
    ui.horizontal(|ui| {
        // Time badge
        let time_text = if let Some(duration) = entry.duration {
            // Calculate end time from start_time + duration
            if let Ok(start) = chrono::NaiveTime::parse_from_str(&entry.start_time, "%H:%M") {
                let end = start + chrono::Duration::minutes(duration as i64);
                format!("⏱ {} - {}", entry.start_time, end.format("%H:%M"))
            } else {
                format!("⏱ {} ({} min)", entry.start_time, duration)
            }
        } else {
            format!("⏱ {}", entry.start_time)
        };

        ui.label(
            egui::RichText::new(time_text)
                .strong()
                .color(egui::Color32::from_rgb(100, 149, 237)), // Cornflower blue
        );
    });

    // Render content as markdown
    if !entry.content.is_empty() {
        ui.add_space(4.0);
        CommonMarkViewer::new().show(ui, cache, &entry.content);
    } else {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("(empty entry)")
                .weak()
                .italics(),
        );
    }
}

