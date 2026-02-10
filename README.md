A simple helper for time tracking

* shows calendar events
* allows creating of diary entries
* shows git commits authored by the user

### Usage

```bash
cargo run
```

### Configuration

Create `~/.config/wdid/config.toml` with the following content:

```toml
theme = "system" # light, dark, system
work_folders = ["list of folders to scan for git repositories"]
work_emails = ["your.email@example.com", "your.other.email@example.com"]

# Add calendar feeds below:
# [[calendars]]
# url = "https://calendar.google.com/calendar/ical/..."
# name = "Work"
# color = "#3b82f6"
# user_email = "you@example.com"  # Your email for this calendar (for attendance status)
```


