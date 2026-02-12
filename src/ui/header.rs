use chrono::{Days, Local};
use eframe::egui::{self, Align, Layout, Sense};

use super::DiaryViewState;
use crate::export::ExportAction;

/// Actions that can be triggered from the header
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HeaderAction {
    #[default]
    None,
    RefreshCalendars,
    RefreshGitCommits,
}

/// Render the header bar with date navigation and search box.
/// Returns a tuple of (HeaderAction, ExportAction) for actions triggered by user interaction.
pub fn render_header(
    ui: &mut egui::Ui,
    state: &mut DiaryViewState,
    has_calendars: bool,
) -> (HeaderAction, ExportAction) {
    let mut action = HeaderAction::None;
    let mut export_action = ExportAction::None;

    ui.horizontal(|ui| {
        if ui.button("Today").clicked() {
            state.current_date = Local::now().date_naive();
        }
        // Left arrow - go to previous day
        if ui.button("◀").clicked() {
            state.current_date = state
                .current_date
                .checked_sub_days(Days::new(1))
                .unwrap_or(state.current_date);
        }

        // Date label with fixed width
        ui.add_sized(
            egui::vec2(120.0, 20.0),
            egui::Label::new(state.current_date.format("%a, %b %d, %Y").to_string()),
        );

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
                    let refresh_btn = ui.button("📅");
                    if refresh_btn.clicked() {
                        action = HeaderAction::RefreshCalendars;
                    }
                    refresh_btn.on_hover_text("Refresh calendar feeds");
                }

                if state.git_refreshing {
                    // Show loading indicator (non-interactive)
                    ui.add(egui::Label::new("⏳").sense(Sense::hover()))
                        .on_hover_text("Refreshing git commits...");
                } else {
                    // Clickable refresh button
                    let refresh_btn = ui.button("💻");
                    if refresh_btn.clicked() {
                        action = HeaderAction::RefreshGitCommits;
                    }
                    refresh_btn.on_hover_text("Refresh git commits");
                }
            }

            // Export menu
            ui.add_space(8.0);
            ui.menu_button("📤 Export", |ui| {
                ui.set_min_width(180.0);

                if ui.button("📋 Today → Clipboard (Markdown)").clicked() {
                    export_action = ExportAction::DayMarkdownClipboard;
                    ui.close();
                }
                if ui.button("💾 Today → File (Markdown)").clicked() {
                    export_action = ExportAction::DayMarkdownFile;
                    ui.close();
                }
                ui.separator();
                if ui.button("📋 Today → Clipboard (JSON)").clicked() {
                    export_action = ExportAction::DayJsonClipboard;
                    ui.close();
                }
                if ui.button("💾 Today → File (JSON)").clicked() {
                    export_action = ExportAction::DayJsonFile;
                    ui.close();
                }
                ui.separator();
                if ui.button("📋 Standup Summary").clicked() {
                    export_action = ExportAction::StandupClipboard;
                    ui.close();
                }
                if ui.button("📋 Weekly Retro").clicked() {
                    export_action = ExportAction::WeeklyRetroClipboard;
                    ui.close();
                }
            });
        });
    });

    (action, export_action)
}

