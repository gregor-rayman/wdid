//! Standup and weekly summary generation.

use crate::db::DiaryEntry;
use chrono::NaiveDate;
use std::collections::BTreeMap;

/// Format a concise standup summary for today's entries.
/// Returns bullet list suitable for team standup.
pub fn format_standup(entries: &[DiaryEntry]) -> String {
    if entries.is_empty() {
        return "No entries for today.".to_string();
    }

    let mut output = String::from("**What I did:**\n");
    for entry in entries {
        let summary = entry.content.lines().next().unwrap_or("").trim();
        let truncated = if summary.len() > 80 {
            format!("{}...", &summary[..77])
        } else {
            summary.to_string()
        };

        if let Some(mins) = entry.duration {
            output.push_str(&format!("- {} ({})\n", truncated, format_duration(mins)));
        } else {
            output.push_str(&format!("- {}\n", truncated));
        }
    }
    output
}

/// Format weekly retro summary grouped by day with totals.
pub fn format_weekly_retro(entries: &[DiaryEntry], week_start: &NaiveDate) -> String {
    let mut by_day: BTreeMap<String, Vec<&DiaryEntry>> = BTreeMap::new();

    for entry in entries {
        by_day.entry(entry.date.clone()).or_default().push(entry);
    }

    let mut output = format!("# Week of {}\n\n", week_start.format("%Y-%m-%d"));
    let mut total_mins = 0;

    for (date, day_entries) in &by_day {
        let day_total: i32 = day_entries.iter().filter_map(|e| e.duration).sum();
        total_mins += day_total;

        let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok();
        let weekday = parsed_date
            .map(|d| d.format("%A").to_string())
            .unwrap_or_default();

        output.push_str(&format!(
            "## {} ({}) — {}\n",
            date,
            weekday,
            format_duration(day_total)
        ));
        for entry in day_entries {
            let summary = entry.content.lines().next().unwrap_or("").trim();
            let dur = entry
                .duration
                .map(|m| format!(" ({})", format_duration(m)))
                .unwrap_or_default();
            output.push_str(&format!("- {}{}\n", summary, dur));
        }
        output.push('\n');
    }

    output.push_str(&format!("**Weekly Total:** {}\n", format_duration(total_mins)));
    output
}

fn format_duration(minutes: i32) -> String {
    let h = minutes / 60;
    let m = minutes % 60;
    match (h, m) {
        (0, m) => format!("{}m", m),
        (h, 0) => format!("{}h", h),
        (h, m) => format!("{}h {}m", h, m),
    }
}
