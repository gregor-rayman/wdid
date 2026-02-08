use serde::Deserialize;

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub calendars: Vec<CalendarFeed>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CalendarFeed {
    pub url: String,
    pub name: Option<String>,
    pub color: Option<String>,
}

