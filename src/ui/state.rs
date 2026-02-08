use chrono::{Local, NaiveDate, NaiveTime, Timelike};

use crate::db::DiaryEntry;

/// Snap a time to the nearest 15-minute interval.
pub fn snap_to_15_minutes(time: NaiveTime) -> NaiveTime {
    let minutes = time.minute();
    let snapped = ((minutes + 7) / 15) * 15;
    let snapped = snapped.min(45); // Cap at :45, don't round to next hour
    NaiveTime::from_hms_opt(time.hour(), snapped, 0).unwrap()
}

/// State for the diary view, tracking the current date and editing context.
pub struct DiaryViewState {
    /// The currently displayed date
    pub current_date: NaiveDate,
    /// ID of the entry currently being edited, if any
    pub editing_entry_id: Option<i64>,
    /// Buffer for the entry being edited
    pub edit_buffer: String,
    /// Current search query
    pub search_query: String,
    /// Search results, if a search has been performed
    pub search_results: Option<Vec<DiaryEntry>>,
}

impl Default for DiaryViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiaryViewState {
    /// Create a new DiaryViewState with today's date.
    pub fn new() -> Self {
        Self {
            current_date: Local::now().date_naive(),
            editing_entry_id: None,
            edit_buffer: String::new(),
            search_query: String::new(),
            search_results: None,
        }
    }
}

