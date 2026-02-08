use egui::{Key, Sense, Ui};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use super::state::DiaryViewState;
use crate::db::DiaryEntry;

/// Actions that can result from entry interaction.
#[derive(Debug, Clone, PartialEq)]
pub enum EntryAction {
    /// No action needed
    None,
    /// Save entry with updated content, start_time, and optional duration
    Save {
        id: i64,
        content: String,
        start_time: String,
        duration: Option<i32>,
    },
    /// Delete entry by ID
    Delete(i64),
}

/// Render a single diary entry with edit/view mode support.
///
/// Returns an `EntryAction` indicating what action should be taken.
pub fn render_entry(
    ui: &mut Ui,
    state: &mut DiaryViewState,
    entry: &DiaryEntry,
    cache: &mut CommonMarkCache,
) -> EntryAction {
    let is_editing = state.editing_entry_id == Some(entry.id);

    if is_editing {
        render_edit_mode(ui, state, entry)
    } else {
        render_view_mode(ui, state, entry, cache)
    }
}

/// Render entry in edit mode with TextEdit for content and time/duration inputs.
fn render_edit_mode(ui: &mut Ui, state: &mut DiaryViewState, entry: &DiaryEntry) -> EntryAction {
    let mut action = EntryAction::None;

    // Time and duration inputs in a horizontal row
    ui.horizontal(|ui| {
        ui.label("Time:");
        ui.add(
            egui::TextEdit::singleline(&mut state.start_time_buffer)
                .desired_width(50.0)
                .hint_text("HH:MM"),
        );

        ui.add_space(16.0);
        ui.label("Duration (min):");
        ui.add(
            egui::TextEdit::singleline(&mut state.duration_buffer)
                .desired_width(40.0)
                .hint_text("—"),
        );
    });

    ui.add_space(4.0);

    // Content TextEdit - multiline
    let response = ui.add(
        egui::TextEdit::multiline(&mut state.edit_buffer)
            .desired_width(f32::INFINITY)
            .min_size(egui::vec2(200.0, 80.0))
            .hint_text("Write your entry..."),
    );

    // Request focus on first frame of editing
    if !state.edit_focus_set {
        response.request_focus();
        state.edit_focus_set = true;
    }

    // Check for save triggers: Escape key or lost focus (click outside)
    let escape_pressed = ui.input(|i| i.key_pressed(Key::Escape));
    let lost_focus = response.lost_focus() && !ui.input(|i| i.key_pressed(Key::Escape));

    if escape_pressed || lost_focus {
        // Parse duration
        let duration: Option<i32> = state
            .duration_buffer
            .trim()
            .parse()
            .ok()
            .filter(|&d: &i32| d > 0);

        // Use updated start_time or fall back to original
        let start_time = if state.start_time_buffer.trim().is_empty() {
            entry.start_time.clone()
        } else {
            state.start_time_buffer.trim().to_string()
        };

        action = EntryAction::Save {
            id: entry.id,
            content: state.edit_buffer.clone(),
            start_time,
            duration,
        };

        // Clear editing state
        state.editing_entry_id = None;
        state.edit_buffer.clear();
        state.start_time_buffer.clear();
        state.duration_buffer.clear();
        state.edit_focus_set = false;
    }

    action
}

/// Render entry in view mode with markdown content.
fn render_view_mode(
    ui: &mut Ui,
    state: &mut DiaryViewState,
    entry: &DiaryEntry,
    cache: &mut CommonMarkCache,
) -> EntryAction {
    let mut action = EntryAction::None;

    // Wrap entire entry in a clickable frame
    let frame_response = egui::Frame::new()
        .inner_margin(4.0)
        .show(ui, |ui| {
            // Time badge
            ui.horizontal(|ui| {
                let time_text = if let Some(duration) = entry.duration {
                    if let Ok(start) =
                        chrono::NaiveTime::parse_from_str(&entry.start_time, "%H:%M")
                    {
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
                        .color(egui::Color32::from_rgb(100, 149, 237)),
                );
            });

            // Render content as markdown
            if !entry.content.is_empty() {
                ui.add_space(4.0);
                CommonMarkViewer::new().show(ui, cache, &entry.content);
            } else {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("(empty entry)").weak().italics());
            }
        });

    // Make the frame clickable
    let response = frame_response.response.interact(Sense::click());

    // Enter edit mode on click
    if response.clicked() {
        state.editing_entry_id = Some(entry.id);
        state.edit_buffer = entry.content.clone();
        state.start_time_buffer = entry.start_time.clone();
        state.duration_buffer = entry.duration.map(|d| d.to_string()).unwrap_or_default();
        state.edit_focus_set = false;
    }

    // Context menu for delete
    response.context_menu(|ui| {
        if ui.button("🗑 Delete").clicked() {
            action = EntryAction::Delete(entry.id);
            ui.close();
        }
    });

    action
}

