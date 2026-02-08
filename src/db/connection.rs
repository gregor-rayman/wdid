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
        "#,
        )?;

        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

