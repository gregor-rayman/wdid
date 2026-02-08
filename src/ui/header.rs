use chrono::Days;
use eframe::egui::{self, Align, Layout};

use super::DiaryViewState;

/// Render the header bar with date navigation and search box.
pub fn render_header(ui: &mut egui::Ui, state: &mut DiaryViewState) {
    ui.horizontal(|ui| {
        // Left arrow - go to previous day
        if ui.button("◀").clicked() {
            state.current_date = state
                .current_date
                .checked_sub_days(Days::new(1))
                .unwrap_or(state.current_date);
        }

        // Date label
        ui.label(state.current_date.format("%A, %B %d, %Y").to_string());

        // Right arrow - go to next day
        if ui.button("▶").clicked() {
            state.current_date = state
                .current_date
                .checked_add_days(Days::new(1))
                .unwrap_or(state.current_date);
        }

        // Spacer to push search to the right
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Search text input
            ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text("🔍 Search...")
                    .desired_width(200.0),
            );
        });
    });
}

