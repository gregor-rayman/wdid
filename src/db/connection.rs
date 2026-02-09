use rusqlite::Connection;
use std::path::Path;

use crate::error::Result;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Create schema
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS diary_entries (
                id INTEGER PRIMARY KEY,
                date TEXT NOT NULL,
                start_time TEXT NOT NULL,
                duration INTEGER,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                event_uid TEXT,
                event_snapshot TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_entries_date ON diary_entries(date);

            CREATE TABLE IF NOT EXISTS calendar_events (
                id INTEGER PRIMARY KEY,
                feed_url TEXT NOT NULL,
                event_uid TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '',
                dtstart_date TEXT NOT NULL,
                dtstart_time TEXT,
                dtend_date TEXT,
                dtend_time TEXT,
                all_day INTEGER DEFAULT 0,
                rrule TEXT,
                status TEXT DEFAULT 'accepted',
                cached_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(feed_url, event_uid, dtstart_date)
            );
            CREATE INDEX IF NOT EXISTS idx_events_date ON calendar_events(dtstart_date);
            CREATE INDEX IF NOT EXISTS idx_events_feed ON calendar_events(feed_url);

            CREATE TABLE IF NOT EXISTS calendar_feeds (
                url TEXT PRIMARY KEY,
                name TEXT,
                color TEXT,
                last_refresh TEXT,
                last_error TEXT
            );
        "#,
        )?;

        // Migration: Add status column to calendar_events if it doesn't exist
        // This handles existing databases that were created before this feature
        let has_status_column: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('calendar_events') WHERE name = 'status'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !has_status_column {
            conn.execute(
                "ALTER TABLE calendar_events ADD COLUMN status TEXT DEFAULT 'accepted'",
                [],
            )?;
        }

        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

