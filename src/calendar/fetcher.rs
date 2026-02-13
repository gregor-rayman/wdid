//! Async calendar feed fetcher with background worker.
//!
//! Spawns a background thread running a tokio runtime that fetches
//! iCal feeds without blocking the UI thread.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::config::CalendarFeed;

/// Commands that can be sent to the calendar worker.
#[derive(Debug)]
pub enum CalendarCommand {
    /// Refresh all configured calendar feeds.
    RefreshAll(Vec<CalendarFeed>),
    // /// Refresh a single feed by URL.
    // RefreshOne(CalendarFeed),
    /// Shutdown the worker thread.
    Shutdown,
}

/// Results sent back from the calendar worker.
#[derive(Debug)]
pub enum CalendarResult {
    /// Successfully fetched feed data.
    FeedData {
        feed_url: String,
        feed_name: Option<String>,
        feed_color: Option<String>,
        data: String,
    },
    /// Error fetching a feed.
    FeedError { feed_url: String, error: String },
    /// All feeds in a RefreshAll batch have been processed.
    RefreshComplete,
    /// Shutdown complete.
    ShutdownComplete,
}

/// Spawns a background worker thread with a tokio runtime.
///
/// Returns a tuple of:
/// - Sender for sending commands to the worker
/// - Receiver for receiving results from the worker
pub fn spawn_calendar_worker() -> (Sender<CalendarCommand>, Receiver<CalendarResult>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<CalendarCommand>();
    let (result_tx, result_rx) = mpsc::channel::<CalendarResult>();

    thread::spawn(move || {
        // Create a new tokio runtime for this thread
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async {
            worker_loop(cmd_rx, result_tx).await;
        });
    });

    (cmd_tx, result_rx)
}

/// Main worker loop that processes commands.
async fn worker_loop(cmd_rx: Receiver<CalendarCommand>, result_tx: Sender<CalendarResult>) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("wdid/0.1")
        .build()
        .expect("Failed to create HTTP client");

    loop {
        // Block waiting for commands (this is fine, we're in a dedicated thread)
        match cmd_rx.recv() {
            Ok(CalendarCommand::RefreshAll(feeds)) => {
                for feed in feeds {
                    fetch_and_send(&client, feed, &result_tx).await;
                }
                let _ = result_tx.send(CalendarResult::RefreshComplete);
            }
            // Ok(CalendarCommand::RefreshOne(feed)) => {
            //     fetch_and_send(&client, feed, &result_tx).await;
            // }
            Ok(CalendarCommand::Shutdown) | Err(_) => {
                let _ = result_tx.send(CalendarResult::ShutdownComplete);
                // Channel closed or shutdown requested, exit loop
                break;
            }
        }
    }
}

/// Fetch a single feed and send the result.
async fn fetch_and_send(
    client: &reqwest::Client,
    feed: CalendarFeed,
    result_tx: &Sender<CalendarResult>,
) {
    let result = fetch_feed(client, &feed).await;

    let calendar_result = match result {
        Ok(data) => CalendarResult::FeedData {
            feed_url: feed.url.clone(),
            feed_name: feed.name.clone(),
            feed_color: feed.color.clone(),
            data,
        },
        Err(e) => CalendarResult::FeedError {
            feed_url: feed.url.clone(),
            error: e.to_string(),
        },
    };

    // Ignore send errors (receiver might be dropped)
    let _ = result_tx.send(calendar_result);
}

/// Fetch a calendar feed via HTTP GET.
async fn fetch_feed(client: &reqwest::Client, feed: &CalendarFeed) -> Result<String, String> {
    let response = client
        .get(&feed.url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))
}

