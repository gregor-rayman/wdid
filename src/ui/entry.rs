use std::collections::HashSet;

use egui::{Color32, CornerRadius, Key, Sense, Ui, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use super::state::DiaryViewState;
use crate::db::DiaryEntry;

/// Default color for linked entries without a known event color.
const DEFAULT_LINK_COLOR: Color32 = Color32::GRAY;

/// Parse a hex color string (e.g., "#4A90D9") to Color32.
fn parse_color(hex: Option<&str>) -> Color32 {
    hex.and_then(|s| {
        let s = s.trim_start_matches('#');
        if s.len() == 6 {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        } else {
            None
        }
    })
    .unwrap_or(DEFAULT_LINK_COLOR)
}

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
    /// Unlink entry from its associated event
    Unlink(i64),
}

/// Render a single diary entry with edit/view mode support.
///
/// `calendar_event_uids` is used to detect orphaned links (entry linked to event that no longer exists).
/// Returns an `EntryAction` indicating what action should be taken.
pub fn render_entry(
    ui: &mut Ui,
    state: &mut DiaryViewState,
    entry: &DiaryEntry,
    cache: &mut CommonMarkCache,
    calendar_event_uids: &HashSet<String>,
) -> EntryAction {
    let is_editing = state.editing_entry_id == Some(entry.id);

    if is_editing {
        render_edit_mode(ui, state, entry)
    } else {
        render_view_mode(ui, state, entry, cache, calendar_event_uids)
    }
}

/// Render entry in edit mode with TextEdit for content and time/duration inputs.
fn render_edit_mode(ui: &mut Ui, state: &mut DiaryViewState, entry: &DiaryEntry) -> EntryAction {
    let mut action = EntryAction::None;

    // Time and duration inputs in a horizontal row
    // Track responses to detect focus on any field
    let mut time_response = None;
    let mut duration_response = None;

    ui.horizontal(|ui| {
        ui.label("Time:");
        time_response = Some(
            ui.add(
                egui::TextEdit::singleline(&mut state.start_time_buffer)
                    .desired_width(50.0)
                    .hint_text("HH:MM"),
            ),
        );

        ui.add_space(16.0);
        ui.label("Duration (min):");
        duration_response = Some(
            ui.add(
                egui::TextEdit::singleline(&mut state.duration_buffer)
                    .desired_width(40.0)
                    .hint_text("—"),
            ),
        );
    });

    ui.add_space(4.0);

    // Content TextEdit - multiline
    let content_response = ui.add(
        egui::TextEdit::multiline(&mut state.edit_buffer)
            .desired_width(f32::INFINITY)
            .min_size(egui::vec2(200.0, 80.0))
            .hint_text("Write your entry..."),
    );

    // Request focus on first frame of editing
    if !state.edit_focus_set {
        content_response.request_focus();
        state.edit_focus_set = true;
    }

    // Check if any of the edit fields have focus
    let any_field_has_focus = content_response.has_focus()
        || time_response.as_ref().map_or(false, |r| r.has_focus())
        || duration_response.as_ref().map_or(false, |r| r.has_focus());

    // Check for save triggers: Escape key or click outside all fields
    let escape_pressed = ui.input(|i| i.key_pressed(Key::Escape));
    let clicked_outside = ui.input(|i| i.pointer.any_click()) && !any_field_has_focus;

    if escape_pressed || clicked_outside {
        // Parse duration
        let duration: Option<i32> = if let Ok(duration_matches) = state
            .duration_buffer
            .split(':')
            .map(|s| s.trim().parse::<i32>())
            .collect::<Result<Vec<_>, _>>()
        {
            if duration_matches.len() >= 2 {
                Some(duration_matches[0] * 60 + duration_matches[1])
            } else if duration_matches.len() == 1 {
                Some(duration_matches[0])
            } else {
                None
            }
        } else {
            None
        };

        // Use updated start_time or fall back to original
        let start_time = if state.start_time_buffer.trim().is_empty() {
            entry.start_time.clone()
        } else {
            if let Ok(start) = chrono::NaiveTime::parse_from_str(
                &state.start_time_buffer.trim().to_string(),
                "%H:%M",
            ) {
                start.format("%H:%M").to_string()
            } else {
                entry.start_time.clone()
            }
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
    calendar_event_uids: &HashSet<String>,
) -> EntryAction {
    let mut action = EntryAction::None;

    // Check if this entry is linked to an event
    let is_linked = entry.event_uid.is_some();
    let is_orphaned = is_linked
        && entry
            .event_uid
            .as_ref()
            .map(|uid| !calendar_event_uids.contains(uid))
            .unwrap_or(false);

    // Determine link color from event snapshot (format: "color:summary")
    // or use default gray for orphaned links
    let link_color = if is_linked {
        entry
            .event_snapshot
            .as_ref()
            .and_then(|s| s.split(':').next())
            .map(|c| parse_color(Some(c)))
            .unwrap_or(DEFAULT_LINK_COLOR)
    } else {
        DEFAULT_LINK_COLOR
    };

    // Wrap entire entry in a frame with optional colored left border
    let frame_response = egui::Frame::new()
        .fill(ui.visuals().widgets.noninteractive.bg_fill)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Vec2::new(0.0, 0.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Colored left border for linked entries
                if is_linked {
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(4.0, 60.0), egui::Sense::hover());
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius {
                            nw: 4,
                            sw: 4,
                            ne: 0,
                            se: 0,
                        },
                        link_color,
                    );
                }

                ui.vertical(|ui| {
                    ui.add_space(4.0);

                    // Orphan warning if applicable
                    if is_orphaned {
                        ui.label(
                            egui::RichText::new("⚠ Event no longer exists")
                                .small()
                                .color(Color32::YELLOW),
                        );
                    }

                    // Time badge
                    ui.horizontal(|ui| {
                        let time_text = if let Some(duration) = entry.duration {
                            if let Ok(start) =
                                chrono::NaiveTime::parse_from_str(&entry.start_time, "%H:%M")
                            {
                                let end = start + chrono::Duration::minutes(duration as i64);
                                format!("⏱ {} - {}", entry.start_time, end.format("%H:%M"))
                            } else if duration > 60 {
                                format!(
                                    "⏱ {} ({}h {}min)",
                                    entry.start_time,
                                    duration / 60,
                                    duration % 60
                                )
                            } else {
                                format!("⏱ {} ({}min)", entry.start_time, duration)
                            }
                        } else {
                            format!("⏱ {}", entry.start_time)
                        };

                        ui.label(
                            egui::RichText::new(time_text)
                                .strong()
                                .color(Color32::from_rgb(100, 149, 237)),
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

                    ui.add_space(4.0);
                });
            });
        });

    // Make the frame clickable
    let response = frame_response.response.interact(Sense::click());

    // Enter edit mode on click
    if response.clicked() {
        state.editing_entry_id = Some(entry.id);
        state.edit_buffer = entry.content.clone();
        state.start_time_buffer = entry.start_time.clone();
        state.duration_buffer = match entry.duration {
            Some(d) if d < 60 => d.to_string(),
            Some(d) => format!("{}:{:02}", d / 60, d % 60),
            None => String::new(),
        };
        state.edit_focus_set = false;
    }

    // Context menu for delete and unlink
    response.context_menu(|ui| {
        if is_linked {
            if ui.button("🔗 Unlink from event").clicked() {
                action = EntryAction::Unlink(entry.id);
                ui.close();
            }
            ui.separator();
        }
        if ui.button("🗑 Delete").clicked() {
            action = EntryAction::Delete(entry.id);
            ui.close();
        }
    });

    action
}
