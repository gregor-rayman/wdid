mod app;
mod config;
mod db;
mod error;
mod paths;

use db::{Database, NewDiaryEntry};

fn main() -> anyhow::Result<()> {
    let paths = paths::AppPaths::new()?;
    paths.ensure_dirs()?;

    let db = Database::open(&paths.database_file)?;

    // Create a test entry
    let entry = NewDiaryEntry {
        date: "2026-02-08".into(),
        start_time: "09:00".into(),
        duration: Some(60),
        content: "Test diary entry".into(),
        event_uid: None,
        event_snapshot: None,
    };

    let id = db.save_entry(&entry)?;
    println!("Saved entry with id: {}", id);

    // Retrieve entries
    let entries = db.get_entries_for_date("2026-02-08")?;
    println!("Found {} entries for 2026-02-08", entries.len());
    for e in &entries {
        println!("  - [{}] {}: {}", e.id, e.start_time, e.content);
    }

    Ok(())
}
