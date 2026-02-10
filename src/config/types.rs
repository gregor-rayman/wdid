use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub calendars: Vec<CalendarFeed>,
    pub theme: String,
    pub work_folders: Vec<String>,
    pub work_emails: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CalendarFeed {
    pub url: String,
    pub name: Option<String>,
    pub color: Option<String>,
    /// User's email address for identifying their ATTENDEE entry in this calendar's events
    pub user_email: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct WindowState {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub x: Option<f32>,  // X11 only, None on Wayland
    pub y: Option<f32>,
}

