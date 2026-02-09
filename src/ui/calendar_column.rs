//! Calendar column rendering for the two-column timeline layout.

use egui::{Color32, CornerRadius, Frame, RichText, Stroke, Ui, Vec2};

use crate::calendar::{CalendarEvent, EventStatus};

/// Default color for events without a configured feed color.
const DEFAULT_EVENT_COLOR: Color32 = Color32::GRAY;

/// Actions that can result from calendar event interaction.
#[derive(Debug, Clone, Default)]
pub enum CalendarAction {
    #[default]
    None,
    /// User wants to add a note linked to this event.
    AddNote {
        event_uid: String,
        summary: String,
        start_time: String,
        feed_color: Option<String>,
    },
}

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
    .unwrap_or(DEFAULT_EVENT_COLOR)
}

/// Render all-day events in a horizontal wrapped layout.
///
/// These appear at the top of the timeline, above the scrollable columns.
pub fn render_all_day_events(ui: &mut Ui, events: &[CalendarEvent]) {
    if events.is_empty() {
        return;
    }

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("All day:").weak());
        for event in events {
            let color = parse_color(event.feed_color.as_deref());
            render_all_day_chip(ui, event, color);
        }
    });
    ui.add_space(8.0);
}

/// Render a single all-day event as a compact chip.
fn render_all_day_chip(ui: &mut Ui, event: &CalendarEvent, color: Color32) {
    // Determine if event should be muted based on status
    let is_cancelled = event.status == EventStatus::Cancelled;
    let is_tentative = event.status == EventStatus::Tentative;

    // Mute colors for cancelled/tentative events
    let display_color = if is_cancelled || is_tentative {
        color.gamma_multiply(0.4)
    } else {
        color
    };

    Frame::new()
        .fill(display_color.gamma_multiply(0.15))
        .stroke(Stroke::new(2.0, display_color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Vec2::new(8.0, 4.0))
        .show(ui, |ui| {
            let text = RichText::new(&event.summary);
            let styled_text = if is_cancelled {
                text.weak().strikethrough()
            } else if is_tentative {
                text.weak()
            } else {
                text
            };
            ui.label(styled_text);
        });
}

/// Render timed calendar events as vertical cards.
///
/// Each event shows time range and summary with a color-coded left border.
/// Returns a `CalendarAction` if user interaction requires one.
pub fn render_calendar_events(ui: &mut Ui, events: &[CalendarEvent]) -> CalendarAction {
    let mut action = CalendarAction::None;

    if events.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("No calendar events").weak());
        });
        return action;
    }

    for event in events {
        let event_action = render_event_card(ui, event);
        if !matches!(event_action, CalendarAction::None) {
            action = event_action;
        }
        ui.add_space(4.0);
    }

    action
}

/// Render a single calendar event card with color-coded left border.
/// Returns a `CalendarAction` if user clicks the 'add note' button.
fn render_event_card(ui: &mut Ui, event: &CalendarEvent) -> CalendarAction {
    let mut action = CalendarAction::None;
    let color = parse_color(event.feed_color.as_deref());
    let time_display = event.time_display();

    // Determine if event should be muted based on status
    let is_cancelled = event.status == EventStatus::Cancelled;
    let is_tentative = event.status == EventStatus::Tentative;

    // Mute colors for cancelled/tentative events
    let display_color = if is_cancelled || is_tentative {
        color.gamma_multiply(0.4)
    } else {
        color
    };

    // Frame with colored left border effect
    // We achieve this by nesting: outer frame with left margin colored, inner content
    Frame::new()
        .fill(ui.visuals().widgets.noninteractive.bg_fill)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Vec2::new(0.0, 0.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Color bar on left
                let (rect, _) = ui.allocate_exact_size(Vec2::new(4.0, 40.0), egui::Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    CornerRadius {
                        nw: 4,
                        sw: 4,
                        ne: 0,
                        se: 0,
                    },
                    display_color,
                );

                // Event content
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    // Time display and add note button
                    ui.horizontal(|ui| {
                        if !time_display.is_empty() {
                            let time_text = RichText::new(&time_display).small().strong();
                            let styled_time = if is_cancelled {
                                time_text.weak().strikethrough()
                            } else if is_tentative {
                                time_text.weak()
                            } else {
                                time_text
                            };
                            ui.label(styled_time);
                        }
                        // Add note button
                        let add_btn = ui.small_button("📝").on_hover_text("Add note for this event");
                        if add_btn.clicked() {
                            action = CalendarAction::AddNote {
                                event_uid: event.event_uid.clone(),
                                summary: event.summary.clone(),
                                start_time: time_display.clone(),
                                feed_color: event.feed_color.clone(),
                            };
                        }
                    });
                    // Summary with status-based styling
                    let summary_text = RichText::new(&event.summary);
                    let styled_summary = if is_cancelled {
                        summary_text.weak().strikethrough()
                    } else if is_tentative {
                        summary_text.weak()
                    } else {
                        summary_text
                    };
                    ui.label(styled_summary);
                    ui.add_space(4.0);
                });
            });
        });

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_valid() {
        assert_eq!(parse_color(Some("#FF0000")), Color32::from_rgb(255, 0, 0));
        assert_eq!(parse_color(Some("#00FF00")), Color32::from_rgb(0, 255, 0));
        assert_eq!(parse_color(Some("#0000FF")), Color32::from_rgb(0, 0, 255));
        assert_eq!(parse_color(Some("#4A90D9")), Color32::from_rgb(74, 144, 217));
    }

    #[test]
    fn test_parse_color_without_hash() {
        assert_eq!(parse_color(Some("FF0000")), Color32::from_rgb(255, 0, 0));
    }

    #[test]
    fn test_parse_color_invalid() {
        assert_eq!(parse_color(Some("#FFF")), DEFAULT_EVENT_COLOR); // Too short
        assert_eq!(parse_color(Some("invalid")), DEFAULT_EVENT_COLOR);
        assert_eq!(parse_color(None), DEFAULT_EVENT_COLOR);
    }
}

