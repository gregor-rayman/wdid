use rusqlite::{params, Row};

use super::Database;
use crate::error::Result;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiaryEntry {
    pub id: i64,
    pub date: String,           // YYYY-MM-DD format
    pub start_time: String,     // HH:MM format
    pub duration: Option<i32>,  // minutes
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub event_uid: Option<String>,
    pub event_snapshot: Option<String>,
}

pub struct NewDiaryEntry {
    pub date: String,
    pub start_time: String,
    pub duration: Option<i32>,
    pub content: String,
    pub event_uid: Option<String>,
    pub event_snapshot: Option<String>,
}

impl Database {
    pub fn save_entry(&self, entry: &NewDiaryEntry) -> Result<i64> {
        self.conn().execute(
            r#"INSERT INTO diary_entries (date, start_time, duration, content, event_uid, event_snapshot)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![
                entry.date,
                entry.start_time,
                entry.duration,
                entry.content,
                entry.event_uid,
                entry.event_snapshot
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    pub fn get_entries_for_date(&self, date: &str) -> Result<Vec<DiaryEntry>> {
        let mut stmt = self.conn().prepare_cached(
            r#"SELECT id, date, start_time, duration, content, created_at, updated_at,
                      event_uid, event_snapshot
               FROM diary_entries WHERE date = ?1
               ORDER BY start_time, created_at"#,
        )?;

        let entries = stmt
            .query_map([date], |row| Ok(DiaryEntry::from_row(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Update an entry with content, start time, and optional duration.
    pub fn update_entry_full(
        &self,
        id: i64,
        content: &str,
        start_time: &str,
        duration: Option<i32>,
    ) -> Result<()> {
        self.conn().execute(
            r#"UPDATE diary_entries
               SET content = ?1, start_time = ?2, duration = ?3, updated_at = datetime('now')
               WHERE id = ?4"#,
            params![content, start_time, duration, id],
        )?;
        Ok(())
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        self.conn().execute("DELETE FROM diary_entries WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Search entries by hashtag (without the # prefix).
    /// Returns entries containing #tag, sorted by date (newest first) then time.
    pub fn search_by_hashtag(&self, tag: &str) -> Result<Vec<DiaryEntry>> {
        let mut stmt = self.conn().prepare_cached(
            r#"SELECT id, date, start_time, duration, content, created_at, updated_at,
                      event_uid, event_snapshot
               FROM diary_entries
               WHERE content LIKE '%#' || ?1 || '%'
               ORDER BY date DESC, start_time"#,
        )?;

        let entries = stmt
            .query_map([tag], |row| Ok(DiaryEntry::from_row(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Search entries by text content (case-insensitive for ASCII).
    /// Returns entries containing the query, sorted by date (newest first) then time.
    pub fn search_by_text(&self, query: &str) -> Result<Vec<DiaryEntry>> {
        let mut stmt = self.conn().prepare_cached(
            r#"SELECT id, date, start_time, duration, content, created_at, updated_at,
                      event_uid, event_snapshot
               FROM diary_entries
               WHERE content LIKE '%' || ?1 || '%'
               ORDER BY date DESC, start_time"#,
        )?;

        let entries = stmt
            .query_map([query], |row| Ok(DiaryEntry::from_row(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Link a diary entry to a calendar event.
    /// Stores the event UID and a snapshot of the event summary for display
    /// even if the original event is later deleted.
    pub fn _link_entry_to_event(
        &self,
        entry_id: i64,
        event_uid: &str,
        event_snapshot: &str,
    ) -> Result<()> {
        self.conn().execute(
            r#"UPDATE diary_entries
               SET event_uid = ?1, event_snapshot = ?2, updated_at = datetime('now')
               WHERE id = ?3"#,
            params![event_uid, event_snapshot, entry_id],
        )?;
        Ok(())
    }

    /// Unlink a diary entry from its associated calendar event.
    /// Preserves the entry content but removes the event association.
    pub fn unlink_entry(&self, entry_id: i64) -> Result<()> {
        self.conn().execute(
            r#"UPDATE diary_entries
               SET event_uid = NULL, event_snapshot = NULL, updated_at = datetime('now')
               WHERE id = ?1"#,
            [entry_id],
        )?;
        Ok(())
    }

    /// Get entries for a date range (inclusive).
    /// Dates in YYYY-MM-DD format. Returns entries sorted by date, then time.
    pub fn get_entries_for_date_range(&self, start: &str, end: &str) -> Result<Vec<DiaryEntry>> {
        let mut stmt = self.conn().prepare_cached(
            r#"SELECT id, date, start_time, duration, content, created_at, updated_at,
                      event_uid, event_snapshot
               FROM diary_entries WHERE date >= ?1 AND date <= ?2
               ORDER BY date, start_time, created_at"#,
        )?;
        let entries = stmt
            .query_map(params![start, end], |row| Ok(DiaryEntry::from_row(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(entries)
    }
}

impl DiaryEntry {
    fn from_row(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            date: row.get(1).unwrap(),
            start_time: row.get(2).unwrap(),
            duration: row.get(3).unwrap(),
            content: row.get(4).unwrap(),
            created_at: row.get(5).unwrap(),
            updated_at: row.get(6).unwrap(),
            event_uid: row.get(7).unwrap(),
            event_snapshot: row.get(8).unwrap(),
        }
    }
}

