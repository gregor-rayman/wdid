use chrono::{NaiveDate, NaiveTime};
use rusqlite::{params, Row};

use super::Database;
use crate::calendar::{CalendarEvent, EventStatus};
use crate::error::Result;

/// Cached feed metadata
#[derive(Debug, Clone)]
pub struct CachedFeed {
    pub url: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub last_refresh: Option<String>,
    pub last_error: Option<String>,
}

impl Database {
    /// Save a calendar event to the cache.
    /// Uses INSERT OR REPLACE to handle the UNIQUE constraint.
    pub fn save_calendar_event(&self, event: &CalendarEvent) -> Result<i64> {
        self.conn().execute(
            r#"INSERT OR REPLACE INTO calendar_events
               (feed_url, event_uid, summary, dtstart_date, dtstart_time,
                dtend_date, dtend_time, all_day, rrule, status)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                event.feed_url,
                event.event_uid,
                event.summary,
                event.dtstart_date.format("%Y-%m-%d").to_string(),
                event.dtstart_time.map(|t| t.format("%H:%M").to_string()),
                event.dtend_date.map(|d| d.format("%Y-%m-%d").to_string()),
                event.dtend_time.map(|t| t.format("%H:%M").to_string()),
                event.all_day as i32,
                event.rrule,
                event.status.as_str(),
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// Get all calendar events for a specific date.
    /// Returns events sorted by all_day status (all-day first), then start time.
    pub fn get_calendar_events_for_date(&self, date: NaiveDate) -> Result<Vec<CalendarEvent>> {
        let mut stmt = self.conn().prepare_cached(
            r#"SELECT e.id, e.feed_url, e.event_uid, e.summary,
                      e.dtstart_date, e.dtstart_time, e.dtend_date, e.dtend_time,
                      e.all_day, e.rrule, e.status, f.name, f.color
               FROM calendar_events e
               LEFT JOIN calendar_feeds f ON e.feed_url = f.url
               WHERE e.dtstart_date = ?1
               ORDER BY e.all_day DESC, e.dtstart_time"#,
        )?;

        let date_str = date.format("%Y-%m-%d").to_string();
        let events = stmt
            .query_map([&date_str], |row| Ok(CalendarEvent::from_row(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(events)
    }

    /// Clear all cached events for a specific feed.
    /// Called before re-caching fresh data from a feed.
    pub fn clear_feed_events(&self, feed_url: &str) -> Result<()> {
        self.conn().execute(
            "DELETE FROM calendar_events WHERE feed_url = ?1",
            [feed_url],
        )?;
        Ok(())
    }

    /// Save or update feed metadata.
    pub fn save_feed(&self, feed: &CachedFeed) -> Result<()> {
        self.conn().execute(
            r#"INSERT OR REPLACE INTO calendar_feeds (url, name, color, last_refresh, last_error)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
            params![
                feed.url,
                feed.name,
                feed.color,
                feed.last_refresh,
                feed.last_error,
            ],
        )?;
        Ok(())
    }

    /// Get feed metadata by URL.
    pub fn _get_feed(&self, url: &str) -> Result<Option<CachedFeed>> {
        let mut stmt = self.conn().prepare_cached(
            r#"SELECT url, name, color, last_refresh, last_error
               FROM calendar_feeds WHERE url = ?1"#,
        )?;

        let mut rows = stmt.query([url])?;
        if let Some(row) = rows.next()? {
            Ok(Some(CachedFeed {
                url: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                last_refresh: row.get(3)?,
                last_error: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }
}

impl CalendarEvent {
    fn from_row(row: &Row) -> Self {
        let dtstart_date_str: String = row.get(4).unwrap();
        let dtstart_time_str: Option<String> = row.get(5).unwrap();
        let dtend_date_str: Option<String> = row.get(6).unwrap();
        let dtend_time_str: Option<String> = row.get(7).unwrap();
        let all_day: i32 = row.get(8).unwrap();
        let status_str: Option<String> = row.get(10).unwrap();

        Self {
            id: row.get(0).unwrap(),
            feed_url: row.get(1).unwrap(),
            event_uid: row.get(2).unwrap(),
            summary: row.get(3).unwrap(),
            dtstart_date: NaiveDate::parse_from_str(&dtstart_date_str, "%Y-%m-%d").unwrap(),
            dtstart_time: dtstart_time_str
                .and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M").ok()),
            dtend_date: dtend_date_str
                .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            dtend_time: dtend_time_str
                .and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M").ok()),
            all_day: all_day != 0,
            rrule: row.get(9).unwrap(),
            status: EventStatus::from_str(status_str.as_deref().unwrap_or("accepted")),
            feed_name: row.get(11).unwrap(),
            feed_color: row.get(12).unwrap(),
        }
    }
}

