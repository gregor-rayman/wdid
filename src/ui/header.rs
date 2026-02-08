use chrono::Days;
use eframe::egui::{self, Align, Layout, Sense};

use super::DiaryViewState;

/// Actions that can be triggered from the header
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HeaderAction {
    #[default]
    None,
    RefreshCalendars,
}

/// Render the header bar with date navigation and search box.
/// Returns an action if user interaction requires one.
pub fn render_header(
    ui: &mut egui::Ui,
    state: &mut DiaryViewState,
    has_calendars: bool,
) -> HeaderAction {
    let mut action = HeaderAction::None;

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

        // Spacer to push search and refresh to the right
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Search text input
            ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text("🔍 Search...")
                    .desired_width(200.0),
            );

            // Refresh button (only show if calendars are configured)
            if has_calendars {
                ui.add_space(8.0);

                // Show warning icon if there are feed errors
                if !state.feed_errors.is_empty() {
                    ui.label("⚠").on_hover_text(format!(
                        "{} feed(s) with errors",
                        state.feed_errors.len()
                    ));
                }

                // Refresh button: show loading state or clickable refresh
                if state.calendar_refreshing {
                    // Show loading indicator (non-interactive)
                    ui.add(egui::Label::new("⏳").sense(Sense::hover()))
                        .on_hover_text("Refreshing calendars...");
                } else {
                    // Clickable refresh button
                    let refresh_btn = ui.button("🔄");
                    if refresh_btn.clicked() {
                        action = HeaderAction::RefreshCalendars;
                    }
                    refresh_btn.on_hover_text("Refresh calendar feeds");
                }
            }
        });
    });

    action
}

