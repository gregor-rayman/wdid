//! JSON export formatting.

use crate::db::DiaryEntry;

/// Format entries as pretty-printed JSON.
pub fn format_entries_json(entries: &[DiaryEntry]) -> String {
    serde_json::to_string_pretty(entries).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}
