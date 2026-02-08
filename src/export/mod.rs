//! Export functionality for diary entries.

pub mod json;
pub mod markdown;
pub mod summary;

pub use json::format_entries_json;
pub use markdown::format_day_markdown;
pub use summary::{format_standup, format_weekly_retro};

/// Actions that can be triggered from the export menu.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ExportAction {
    #[default]
    None,
    /// Export current day to clipboard as Markdown
    DayMarkdownClipboard,
    /// Export current day to file as Markdown
    DayMarkdownFile,
    /// Export current day to clipboard as JSON
    DayJsonClipboard,
    /// Export current day to file as JSON
    DayJsonFile,
    /// Copy standup summary to clipboard
    StandupClipboard,
    /// Copy weekly retro to clipboard
    WeeklyRetroClipboard,
}

