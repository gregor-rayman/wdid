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
    /// Buffer for the entry content being edited
    pub edit_buffer: String,
    /// Buffer for the start time being edited (HH:MM format)
    pub start_time_buffer: String,
    /// Buffer for the duration being edited (minutes as string)
    pub duration_buffer: String,
    /// Track if focus has been set for the current edit session
    pub edit_focus_set: bool,
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
            start_time_buffer: String::new(),
            duration_buffer: String::new(),
            edit_focus_set: false,
            search_query: String::new(),
            search_results: None,
        }
    }
}

