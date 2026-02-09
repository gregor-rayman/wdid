use chrono::{NaiveDate, NaiveTime};

/// Status of a calendar event.
///
/// This represents the event's STATUS property from iCal (CONFIRMED, TENTATIVE, CANCELLED).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub enum EventStatus {
    /// Event is confirmed
    #[default]
    Confirmed,
    /// Event has been cancelled
    Cancelled,
    /// Event is tentatively scheduled
    Tentative,
}

impl EventStatus {
    /// Convert to string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Confirmed => "confirmed",
            EventStatus::Cancelled => "cancelled",
            EventStatus::Tentative => "tentative",
        }
    }

    /// Parse from string (database retrieval).
    pub fn from_str(s: &str) -> Self {
        match s {
            "cancelled" | "declined" => EventStatus::Cancelled,
            "tentative" => EventStatus::Tentative,
            // Default for "confirmed", "accepted", or unknown
            _ => EventStatus::Confirmed,
        }
    }
}

/// A calendar event (either from cache or freshly parsed)
#[derive(Debug, Clone, serde::Serialize)]
pub struct CalendarEvent {
    pub id: Option<i64>,            // DB id, None if not yet saved
    pub feed_url: String,
    pub event_uid: String,          // iCal UID
    pub summary: String,
    pub dtstart_date: NaiveDate,    // Date portion
    pub dtstart_time: Option<NaiveTime>, // None for all-day
    pub dtend_date: Option<NaiveDate>,
    pub dtend_time: Option<NaiveTime>,
    pub all_day: bool,
    pub rrule: Option<String>,      // Original RRULE for re-expansion
    pub feed_name: Option<String>,  // From config
    pub feed_color: Option<String>, // From config, e.g. "#4A90D9"
    pub status: EventStatus,        // Event's status (Confirmed, Tentative, Cancelled)
}

impl CalendarEvent {
    /// Format display time range (e.g., "9:00-13:00" or "All day")
    pub fn time_display(&self) -> String {
        if self.all_day {
            "All day".to_string()
        } else if let Some(start) = self.dtstart_time {
            if let Some(end) = self.dtend_time {
                format!("{} - {}", start.format("%H:%M"), end.format("%H:%M"))
            } else {
                start.format("%H:%M").to_string()
            }
        } else {
            String::new()
        }
    }
}

