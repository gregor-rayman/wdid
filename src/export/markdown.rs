//! Markdown export formatting.

use crate::db::DiaryEntry;
use chrono::NaiveDate;

/// Format a single day's entries as Markdown.
/// Returns a human-readable document with date header and entry list.
pub fn format_day_markdown(date: &NaiveDate, entries: &[DiaryEntry]) -> String {
    let mut output = String::new();

    // Header with date and day name
    let weekday = date.format("%A").to_string();
    output.push_str(&format!("# {} ({})\n\n", date.format("%Y-%m-%d"), weekday));

    if entries.is_empty() {
        output.push_str("*No entries*\n");
        return output;
    }

    for entry in entries {
        output.push_str(&format_entry_markdown(entry));
        output.push('\n');
    }

    output
}

/// Format a single entry as Markdown.
fn format_entry_markdown(entry: &DiaryEntry) -> String {
    let mut line = format!("## {}", entry.start_time);

    // Add duration if present
    if let Some(mins) = entry.duration {
        let end_time = add_minutes_to_time(&entry.start_time, mins);
        line.push_str(&format!(" - {} ({})", end_time, format_duration(mins)));
    }

    // Add linked event name if present
    if let Some(ref snapshot) = entry.event_snapshot {
        if let Some(summary) = snapshot.split(':').nth(1) {
            line.push_str(&format!(" — {}", summary));
        }
    }

    line.push('\n');
    line.push_str(&entry.content);
    line.push('\n');

    line
}

/// Format duration as human-readable string (e.g., "1h 30m").
fn format_duration(minutes: i32) -> String {
    let h = minutes / 60;
    let m = minutes % 60;
    match (h, m) {
        (0, m) => format!("{}m", m),
        (h, 0) => format!("{}h", h),
        (h, m) => format!("{}h {}m", h, m),
    }
}

/// Add minutes to a time string (HH:MM) and return new time string.
fn add_minutes_to_time(time: &str, minutes: i32) -> String {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return time.to_string();
    }

    let h: i32 = parts[0].parse().unwrap_or(0);
    let m: i32 = parts[1].parse().unwrap_or(0);
    let total = h * 60 + m + minutes;
    format!("{:02}:{:02}", (total / 60) % 24, total % 60)
}
