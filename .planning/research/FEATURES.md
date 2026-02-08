# Feature Research

**Domain:** Daily work tracking / diary / journal applications  
**Researched:** 2026-02-08  
**Confidence:** MEDIUM (verified via multiple sources: Zapier, Rosebud, Reflection.app, Stack Overflow Blog)

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete or broken.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Day Navigation** | Users need to move between days easily | LOW | Already planned: single-day view with navigation |
| **Entry Creation with Timestamp** | Core journaling function | LOW | Already planned: entries with start time |
| **Data Persistence** | Entries must survive app restart | LOW | Already planned: SQLite storage |
| **Text Formatting (Markdown)** | Modern users expect rich text | MEDIUM | Already planned: markdown formatting |
| **Search/Find** | Users need to locate past entries | MEDIUM | Already planned: hashtag search |
| **Quick Entry** | Must be fast to add a note—friction kills habit | LOW | Should be <3 clicks to log entry |
| **Date/Time on Entries** | Context for when work happened | LOW | Already planned: start time, optional duration |
| **Basic Export** | Users need to get data out (PDF, plain text, markdown) | MEDIUM | Critical for trust—avoids lock-in anxiety |
| **Keyboard Shortcuts** | Desktop apps need keyboard-first experience | LOW | Essential for power users; common shortcuts expected |
| **Auto-Save / Draft Protection** | Never lose work in progress | LOW | Table stakes for any text-based app |
| **System Tray/Background Running** | Always-available quick access | LOW | Already planned: system tray with show/hide |
| **Configuration Persistence** | Settings survive restart | LOW | Already planned: TOML config |

### Differentiators (Competitive Advantage)

Features that set wdid apart. Not required, but valuable for target audience (developers tracking daily work).

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Calendar Integration (iCal)** | Context of scheduled vs actual work | MEDIUM | Already planned—this IS the differentiator |
| **Two-Column Timeline** | Side-by-side calendar events + diary = unique | MEDIUM | Already planned—core UI innovation |
| **Linking Entries to Events** | Connect diary to what was scheduled | MEDIUM | Already planned—bridges planning vs reality |
| **Standup/Retro Summary Export** | Generate "what I did" for meetings | MEDIUM | Huge value for developers: standup prep in 1 click |
| **Daily/Weekly Summary Generation** | Auto-summarize for reports or reviews | MEDIUM | Performance review gold; patterns from work log |
| **Hashtag-Based Organization** | Fast categorization without folders | LOW | Already planned; simple but effective |
| **"What did I work on?" Quick View** | Dashboard of recent activity | MEDIUM | Useful for context switching, time tracking |
| **Time Range Filtering** | View work by week/month/project | MEDIUM | Good for retrospectives and billing |
| **Dark Mode / Theme Support** | Developer expectations for desktop apps | LOW | Standard for Linux desktop apps now |
| **Local-First / Offline-First** | Privacy-conscious users; no cloud dependency | LOW | Already implied—SQLite local storage |
| **Multiple iCal Feed Support** | Work + personal calendars combined | LOW | Common need: multiple calendars |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems for this specific domain.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Cloud Sync** | "Access from anywhere" | Complexity explosion; privacy concerns; syncs personal work data | Focus on export/backup; local-first with manual backup |
| **Mobile App** | "Log on the go" | Desktop focus is the strength; mobile competes with existing apps | Stay desktop-native; export to mobile-readable format if needed |
| **AI Summarization** | "Make me look productive" | Adds dependency; privacy concern with work data; scope creep | Manual export + user applies their own AI if desired |
| **Team/Sharing Features** | "Share with manager" | Privacy-first tool becomes collaboration tool; different product | Export formats suitable for sharing (markdown, PDF) |
| **Mood Tracking** | Popular in diary apps | Wrong domain—this is work tracking, not personal journaling | Hashtags can capture energy/mood if user wants |
| **Gamification/Streaks** | "Motivate me to journal" | Work log ≠ habit app; patronizing for professionals | Optional reminder in system tray; no guilt mechanics |
| **Photo/Multimedia Attachments** | Day One popularized this | Work logs are text-centric; adds complexity; storage bloat | Allow paste of images as markdown links if needed |
| **Calendar Event Editing** | "Edit events from here" | Read-only calendar is correct scope; editing = calendar app | Link to calendar app; stay read-only |
| **Notifications/Reminders** | "Remind me to log" | Desktop app; user opens when needed; nagging breaks flow | Gentle system tray indicator at most |
| **Real-time Sync** | Multi-device access | Over-engineering for personal tool; conflict resolution nightmare | Export + manual transfer |

## Feature Dependencies

```
[Calendar Integration] 
    └──enables──> [Linking Entries to Events]
                      └──enables──> [Gap Analysis: scheduled vs done]

[Entry Creation with Timestamp]
    └──enables──> [Standup Summary Export]
    └──enables──> [Search/Find]

[Markdown Formatting]
    └──enables──> [Export (PDF, Markdown)]

[Hashtag Search]
    └──enables──> [Tag-based Filtering]
    └──enables──> [Weekly/Monthly Roll-up by Tag]

[SQLite Storage]
    └──enables──> [Search/Find]
    └──enables──> [Export]
    └──enables──> [Backup/Restore]
```

### Dependency Notes

- **Calendar Integration enables Linking:** Without calendar data, there's nothing to link to
- **Entry timestamps enable summaries:** Need temporal data to generate "what did I do today/this week"
- **Markdown enables rich export:** Export formats depend on structured text content
- **SQLite enables everything:** Local persistence is foundation for all features

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the core concept.

- [ ] Single-day view with day navigation — core UI
- [ ] Diary entry creation (timestamp, duration optional, markdown) — core function
- [ ] Calendar event display from single iCal feed — differentiating feature
- [ ] Link entries to calendar events — unique value proposition
- [ ] Basic search (hashtags at minimum) — discoverability
- [ ] System tray with show/hide — desktop integration
- [ ] SQLite persistence — data safety
- [ ] TOML configuration — customization
- [ ] Basic export (markdown/plain text) — data freedom

### Add After Validation (v1.x)

Features to add once core is working and validated.

- [ ] Multiple iCal feed support — when users request it
- [ ] Standup summary export — when users confirm use case
- [ ] Dark mode — expected enhancement
- [ ] Keyboard shortcuts — power user enablement
- [ ] PDF export — professional output format
- [ ] Weekly/monthly view or roll-up — when daily view is solid

