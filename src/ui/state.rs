use std::collections::HashMap;

use chrono::{DateTime, Local, NaiveDate, NaiveTime, Timelike};

use crate::calendar::CalendarEvent;
use crate::db::DiaryEntry;

/// Which column is being hovered (for scroll priority)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Column {
    Calendar,
    Diary,
}

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
    /// Previous search query (for change detection)
    pub prev_search_query: String,
    /// Search results, if a search has been performed
    pub search_results: Option<Vec<DiaryEntry>>,

    /// Calendar events for the current date (timed events)
    pub calendar_events: Vec<CalendarEvent>,
    /// All-day events for the current date (separate for header display)
    pub all_day_events: Vec<CalendarEvent>,
    /// Track if calendar refresh is in progress
    pub calendar_refreshing: bool,
    /// Per-feed error messages (url -> error)
    pub feed_errors: HashMap<String, String>,
    /// Per-feed last refresh times (url -> timestamp)
    pub feed_last_refresh: HashMap<String, DateTime<Local>>,

    /// Synchronized scroll offset for two-column layout (y-axis)
    pub scroll_offset: f32,
    /// Track which column is being hovered (for scroll priority)
    pub hovered_column: Option<Column>,
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
            prev_search_query: String::new(),
            search_results: None,
            calendar_events: Vec::new(),
            all_day_events: Vec::new(),
            calendar_refreshing: false,
            feed_errors: HashMap::new(),
            feed_last_refresh: HashMap::new(),
            scroll_offset: 0.0,
            hovered_column: None,
        }
    }

    /// Check if search query has changed and update tracking.
    /// Returns true if search should be performed.
    pub fn search_changed(&mut self) -> bool {
        if self.search_query != self.prev_search_query {
            self.prev_search_query = self.search_query.clone();
            true
        } else {
            false
        }
    }
}

