mod types;
pub use types::{CalendarFeed, Config, WindowState};

use std::fs;
use std::path::Path;

pub enum ConfigResult {
    Loaded(Config),
    Created(Config),      // first run, created default config
    ParseError(String),   // user-friendly parse error
}

const DEFAULT_CONFIG: &str = r##"# wdid configuration

# Your email address (used to identify your attendance status in calendar events)
# user_email = "you@example.com"

# Add calendar feeds below:
# [[calendars]]
# url = "https://calendar.google.com/calendar/ical/..."
# name = "Work"
# color = "#3b82f6"
"##;

pub fn load_config(path: &Path) -> ConfigResult {
    if !path.exists() {
        // First run: create config file with examples
        if let Err(e) = fs::write(path, DEFAULT_CONFIG) {
            return ConfigResult::ParseError(format!("Could not create config file: {}", e));
        }
        return ConfigResult::Created(Config::default());
    }

    match fs::read_to_string(path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => ConfigResult::Loaded(config),
            Err(e) => ConfigResult::ParseError(format!(
                "Config file has errors at line {}: {}",
                e.span().map(|s| s.start).unwrap_or(0),
                e.message()
            )),
        },
        Err(e) => ConfigResult::ParseError(format!("Could not read config file: {}", e)),
    }
}

/// Load window state from file. Returns default if file is missing or invalid.
pub fn load_window_state(path: &Path) -> WindowState {
    if !path.exists() {
        return WindowState::default();
    }

    match fs::read_to_string(path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => WindowState::default(),
    }
}

/// Save window state to file.
pub fn save_window_state(path: &Path, state: &WindowState) -> std::io::Result<()> {
    let content = toml::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, content)
}

