use std::collections::HashSet;

use egui::{Align, Layout, ScrollArea, Ui};
use egui_commonmark::CommonMarkCache;

use super::calendar_column::{render_all_day_events, render_calendar_events, CalendarAction};
use super::entry::{render_entry, EntryAction};
use super::state::{Column, DiaryViewState};
use crate::calendar::CalendarEvent;
use crate::db::{DiaryEntry, GitCommit};
use crate::ui::git_commit::render_git_commit;

/// Result of timeline rendering, containing any pending actions.
#[derive(Debug, Clone, Default)]
pub struct TimelineActions {
    /// Entry to save (id, content, start_time, duration)
    pub save: Option<(i64, String, String, Option<i32>)>,
    /// Entry to delete
    pub delete: Option<i64>,
    /// Entry to unlink from event
    pub unlink: Option<i64>,
    /// Calendar action (add note to event)
    pub calendar_action: CalendarAction,
}

/// Render the timeline view showing entries.
///
/// When `is_search_mode` is true, shows single-column search results with date prefixes.
/// Otherwise, shows two-column layout: calendar events left, diary entries right.
/// Returns `TimelineActions` with any pending save/delete operations.
pub fn render_timeline(
    ui: &mut Ui,
    entries: &[DiaryEntry],
    calendar_events: &[CalendarEvent],
    all_day_events: &[CalendarEvent],
    git_commits: &[GitCommit],
    state: &mut DiaryViewState,
    cache: &mut CommonMarkCache,
    is_search_mode: bool,
    git_tag_regex: &str,
) -> TimelineActions {
    //ui.ctx().set_pixels_per_point(2.0);
    let mut actions = TimelineActions::default();

    // Build a set of current calendar event UIDs for orphan detection
    let calendar_event_uids: HashSet<String> = calendar_events
        .iter()
        .chain(all_day_events.iter())
        .map(|e| e.event_uid.clone())
        .collect();

    // In search mode, use single-column layout
    if is_search_mode {
        return render_search_results(ui, entries, state, cache, &calendar_event_uids);
    }

    // Check for completely empty state
    if entries.is_empty() && calendar_events.is_empty() && git_commits.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(egui::RichText::new("No entries for this date").weak());
            ui.add_space(10.0);
            ui.label("Press Ctrl+N to create a new entry");
        });
        return actions;
    }

    // Two-column layout with synchronized scrolling
    let initial_offset = state.scroll_offset;

    // Three-column layout: 25% calendar, 50% diary, 25% git
    let available_width = ui.available_width();
    let spacing = ui.spacing().item_spacing.x;
    let total_spacing = spacing * 2.0; // spacing between 3 columns
    let usable_width = available_width - total_spacing;
    let left_width = usable_width * 0.25;
    let center_width = usable_width * 0.50;
    let right_width = usable_width * 0.25;
    let available_height = ui.available_height();

    let mut calendar_offset = initial_offset;
    let mut diary_offset = initial_offset;
    let mut git_commit_offset = initial_offset;

    ui.horizontal(|ui| {
        // Left column: Calendar events (25%)
        ui.allocate_ui_with_layout(
            egui::vec2(left_width, available_height),
            Layout::top_down(Align::Min),
            |ui| {
                let column_rect = ui.available_rect_before_wrap();
                if ui.rect_contains_pointer(column_rect) {
                    state.hovered_column = Some(Column::Calendar);
                }

                let response = ScrollArea::vertical()
                    .id_salt("calendar_scroll")
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(initial_offset)
                    .show(ui, |ui| {
                        render_all_day_events(ui, all_day_events);
                        let cal_action = render_calendar_events(ui, calendar_events, &state.current_date);
                        if !matches!(cal_action, CalendarAction::None) {
                            actions.calendar_action = cal_action;
                        }
                    });
                calendar_offset = response.state.offset.y;
            },
        );

        // Center column: Diary entries (50%)
        ui.allocate_ui_with_layout(
            egui::vec2(center_width, available_height),
            Layout::top_down(Align::Min),
            |ui| {
                let column_rect = ui.available_rect_before_wrap();
                if ui.rect_contains_pointer(column_rect) {
                    state.hovered_column = Some(Column::Diary);
                }

                let response = ScrollArea::vertical()
                    .id_salt("diary_scroll")
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(initial_offset)
                    .show(ui, |ui| {
                        render_diary_entries(
                            ui,
                            entries,
                            state,
                            cache,
                            &mut actions,
                            &calendar_event_uids,
                        );
                    });
                diary_offset = response.state.offset.y;
            },
        );

        // Right column: Git commits (25%)
        ui.allocate_ui_with_layout(
            egui::vec2(right_width, available_height),
            Layout::top_down(Align::Min),
            |ui| {
                let column_rect = ui.available_rect_before_wrap();
                if ui.rect_contains_pointer(column_rect) {
                    state.hovered_column = Some(Column::Git);
                }

                let response = ScrollArea::vertical()
                    .id_salt("git_scroll")
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(initial_offset)
                    .show(ui, |ui| {
                        render_git_commits(
                            ui,
                            git_commits,
                            cache,
                            &git_tag_regex,
                        );
                    });
                git_commit_offset = response.state.offset.y;
            },
        );
    });

    // Synchronize scroll: use the offset from the hovered/active column
    let new_offset = match state.hovered_column {
        Some(Column::Calendar) => calendar_offset,
        Some(Column::Diary) => diary_offset,
        Some(Column::Git) => git_commit_offset,
        None => {
            if (calendar_offset - initial_offset).abs() > 0.5 {
                calendar_offset
            } else if (diary_offset - initial_offset).abs() > 0.5 {
                diary_offset
            } else {
                initial_offset
            }
        }
    };
    state.scroll_offset = new_offset;

    actions
}

/// Render search results in a single-column layout.
fn render_search_results(
    ui: &mut Ui,
    entries: &[DiaryEntry],
    state: &mut DiaryViewState,
    cache: &mut CommonMarkCache,
    calendar_event_uids: &HashSet<String>,
) -> TimelineActions {
    let mut actions = TimelineActions::default();

    if entries.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(egui::RichText::new("No matching entries").weak());
            ui.add_space(10.0);
            ui.label("Try a different search term");
        });
        return actions;
    }

    // Show search result count
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "🔍 {} result{}",
                entries.len(),
                if entries.len() == 1 { "" } else { "s" }
            ))
            .weak(),
        );
    });
    ui.add_space(8.0);

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut prev_date: Option<&str> = None;

            for entry in entries {
                // Show date headers when date changes
                if prev_date != Some(&entry.date) {
                    if prev_date.is_some() {
                        ui.add_space(12.0);
                        ui.separator();
                    }
                    ui.add_space(8.0);
                    let date_label = format_date_label(&entry.date);
                    ui.label(egui::RichText::new(date_label).strong());
                    prev_date = Some(&entry.date);
                }

                ui.add_space(8.0);
                let action = render_entry(ui, state, entry, cache, calendar_event_uids);

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
                    EntryAction::Unlink(id) => {
                        actions.unlink = Some(id);
                    }
                    EntryAction::None => {}
                }
            }

            ui.add_space(16.0);
        });

    actions
}

/// Render diary entries in a scrollable column.
fn render_diary_entries(
    ui: &mut Ui,
    entries: &[DiaryEntry],
    state: &mut DiaryViewState,
    cache: &mut CommonMarkCache,
    actions: &mut TimelineActions,
    calendar_event_uids: &HashSet<String>,
) {
    if entries.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("No diary entries").weak());
            ui.add_space(10.0);
            ui.label("Press Ctrl+N to create one");
        });
        return;
    }

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
        let action = render_entry(ui, state, entry, cache, calendar_event_uids);

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
            EntryAction::Unlink(id) => {
                actions.unlink = Some(id);
            }
            EntryAction::None => {}
        }

        prev_time = Some(&entry.start_time);
    }

    ui.add_space(16.0);
}
/// Render diary entries in a scrollable column.
fn render_git_commits(
    ui: &mut Ui,
    entries: &[GitCommit],
    cache: &mut CommonMarkCache,
    git_tag_regex: &str,
) {
    if entries.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("No Git commits").weak());
        });
        return;
    }

    // Extract tags from git commit messages using the `git_tag_regex`. Display the unique tags from
    // all commits in a row above the commit messages.
    if !git_tag_regex.is_empty() {
        if let Ok(re) = regex::Regex::new(git_tag_regex) {
            let mut tags: Vec<String> = entries
                .iter()
                .flat_map(|e| re.find_iter(&e.description).map(|m| m.as_str().to_string()))
                .collect();
            tags.sort();
            tags.dedup();

            if !tags.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    for tag in &tags {
                        ui.label(egui::RichText::new(tag).strong());
                    }
                });
                ui.add_space(4.0);
            }
        }
    }

    for entry in entries {
        ui.add_space(4.0);
        render_git_commit(ui, entry, cache);
    }

    ui.add_space(16.0);
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

