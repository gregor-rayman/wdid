//! Export functionality for diary entries.

use arboard::Clipboard;
use rfd::FileDialog;
use std::path::PathBuf;

pub mod json;
pub mod markdown;
pub mod summary;

pub use json::format_entries_json;
pub use markdown::format_day_markdown;
pub use summary::{format_standup, format_weekly_retro};

/// Copy text to system clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    Clipboard::new()
        .and_then(|mut cb| cb.set_text(text))
        .map_err(|e| e.to_string())
}

/// Show file save dialog and write content. Returns path if saved.
pub fn save_to_file(
    content: &str,
    default_name: &str,
    filter_name: &str,
    filter_ext: &str,
) -> Option<PathBuf> {
    let path = FileDialog::new()
        .set_file_name(default_name)
        .add_filter(filter_name, &[filter_ext])
        .save_file()?;
    std::fs::write(&path, content).ok()?;
    Some(path)
}

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

