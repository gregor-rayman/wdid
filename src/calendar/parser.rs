//! iCal parsing with RRULE expansion

use anyhow::{anyhow, Result};
use calcard::icalendar::{
    ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarProperty, ICalendarStatus,
    ICalendarValue,
};
use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Tz as IanaTz;
use rrule::{RRuleSet, Tz};

use super::types::{CalendarEvent, EventStatus};

/// Parse iCal data into CalendarEvents, expanding recurrences within date range.
///
/// # Arguments
/// * `ical_data` - Raw iCal string
/// * `feed_url` - URL of the feed (for tracking)
/// * `range_start` - Start of date range for recurrence expansion
/// * `range_end` - End of date range for recurrence expansion
/// * `feed_name` - Optional display name for the feed
/// * `feed_color` - Optional color for the feed (e.g., "#4A90D9")
pub fn parse_ical(
    ical_data: &str,
    feed_url: &str,
    range_start: NaiveDate,
    range_end: NaiveDate,
    feed_name: Option<String>,
    feed_color: Option<String>,
) -> Result<Vec<CalendarEvent>> {
    let calendar = ICalendar::parse(ical_data)
        .map_err(|e| anyhow!("Failed to parse iCal: {:?}", e))?;

    let mut events = Vec::new();

    for component in &calendar.components {
        if component.component_type != ICalendarComponentType::VEvent {
            continue;
        }

        // Extract basic event properties
        let uid = get_text_property(component, &ICalendarProperty::Uid)
            .unwrap_or_else(|| "unknown".to_string());
        let summary = get_text_property(component, &ICalendarProperty::Summary)
            .unwrap_or_else(|| "(No title)".to_string());

        // Parse DTSTART
        let (dtstart_date, dtstart_time, all_day) = match parse_dtstart(component) {
            Some(result) => result,
            None => continue, // Skip events without DTSTART
        };

        // Parse DTEND (optional)
        let (dtend_date, dtend_time) = parse_dtend(component).unwrap_or((None, None));

        // Parse event STATUS property
        let status = parse_event_status(component);

        // Check for RRULE
        let rrule_str = get_rrule_string(component);

        if let Some(ref rrule) = rrule_str {
            // Expand recurring events
            let occurrences = expand_rrule(
                rrule,
                dtstart_date,
                dtstart_time,
                range_start,
                range_end,
            )?;

            for occurrence_date in occurrences {
                events.push(CalendarEvent {
                    id: None,
                    feed_url: feed_url.to_string(),
                    event_uid: uid.clone(),
                    summary: summary.clone(),
                    dtstart_date: occurrence_date,
                    dtstart_time,
                    dtend_date: dtend_date.map(|_| occurrence_date), // Same-day end for recurring
                    dtend_time,
                    all_day,
                    rrule: Some(rrule.clone()),
                    feed_name: feed_name.clone(),
                    feed_color: feed_color.clone(),
                    status,
                });
            }
        } else {
            // Single event - check if within range
            if dtstart_date >= range_start && dtstart_date <= range_end {
                events.push(CalendarEvent {
                    id: None,
                    feed_url: feed_url.to_string(),
                    event_uid: uid,
                    summary,
                    dtstart_date,
                    dtstart_time,
                    dtend_date,
                    dtend_time,
                    all_day,
                    rrule: None,
                    feed_name: feed_name.clone(),
                    feed_color: feed_color.clone(),
                    status,
                });
            }
        }
    }

    Ok(events)
}

/// Get a text property value from a component
fn get_text_property(component: &ICalendarComponent, prop: &ICalendarProperty) -> Option<String> {
    component.property(prop).and_then(|entry| {
        entry.values.first().and_then(|v| {
            if let ICalendarValue::Text(text) = v {
                Some(text.clone())
            } else {
                None
            }
        })
    })
}

/// Parse the STATUS property from an iCal event.
///
/// The STATUS property indicates the overall status of the event:
/// - CONFIRMED: The event is confirmed
/// - TENTATIVE: The event is tentatively scheduled
/// - CANCELLED: The event has been cancelled
///
/// If no STATUS is present, defaults to Confirmed.
fn parse_event_status(component: &ICalendarComponent) -> EventStatus {
    match component.status() {
        Some(ICalendarStatus::Cancelled) => EventStatus::Cancelled,
        Some(ICalendarStatus::Tentative) => EventStatus::Tentative,
        // CONFIRMED is explicit, but also default for missing STATUS
        Some(ICalendarStatus::Confirmed) | None => EventStatus::Confirmed,
        // Other status values (NeedsAction, Completed, etc. are for TODOs)
        // treat them as confirmed for events
        _ => EventStatus::Confirmed,
    }
}

/// Parse DTSTART from component, returns (date, time, is_all_day)
/// Time is converted to local timezone if timezone info is available.
fn parse_dtstart(component: &ICalendarComponent) -> Option<(NaiveDate, Option<NaiveTime>, bool)> {
    let entry = component.property(&ICalendarProperty::Dtstart)?;
    let tzid = entry.tz_id();
    entry
        .values
        .first()
        .and_then(|v| parse_datetime_value_with_tz(v, tzid))
}

/// Parse DTEND from component
/// Time is converted to local timezone if timezone info is available.
fn parse_dtend(component: &ICalendarComponent) -> Option<(Option<NaiveDate>, Option<NaiveTime>)> {
    let entry = component.property(&ICalendarProperty::Dtend)?;
    let tzid = entry.tz_id();
    let (date, time, _) = entry
        .values
        .first()
        .and_then(|v| parse_datetime_value_with_tz(v, tzid))?;
    Some((Some(date), time))
}

/// Parse a datetime value into (date, time, is_all_day)
/// If timezone info is available, converts to local timezone.
fn parse_datetime_value_with_tz(
    value: &ICalendarValue,
    tzid: Option<&str>,
) -> Option<(NaiveDate, Option<NaiveTime>, bool)> {
    if let ICalendarValue::PartialDateTime(pdt) = value {
        // PartialDateTime has optional year, month, day fields
        let year = pdt.year?;
        let month = pdt.month?;
        let day = pdt.day?;
        let date = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)?;

        // Check if time is present (hour/minute/second)
        if pdt.hour.is_some() {
            let naive_time = NaiveTime::from_hms_opt(
                pdt.hour.unwrap_or(0) as u32,
                pdt.minute.unwrap_or(0) as u32,
                pdt.second.unwrap_or(0) as u32,
            )?;
            let naive_dt = NaiveDateTime::new(date, naive_time);

            // Convert to local timezone
            let local_dt = convert_to_local(naive_dt, pdt.tz_hour, pdt.tz_minute, pdt.tz_minus, tzid);
            Some((local_dt.date_naive(), Some(local_dt.time()), false))
        } else {
            // All-day event - no timezone conversion needed
            Some((date, None, true))
        }
    } else {
        None
    }
}

/// Convert a naive datetime to local timezone given timezone info.
///
/// Priority:
/// 1. If tz_hour is Some(0) and tz_minute is Some(0) or None, it's UTC ('Z' suffix)
/// 2. If tz_hour/tz_minute are set, use the fixed offset
/// 3. If tzid is provided, parse as IANA timezone name
/// 4. Otherwise, assume the time is already in local timezone (floating time)
fn convert_to_local(
    naive_dt: NaiveDateTime,
    tz_hour: Option<u8>,
    tz_minute: Option<u8>,
    tz_minus: bool,
    tzid: Option<&str>,
) -> DateTime<Local> {
    // Check for explicit UTC offset in the PartialDateTime
    if let Some(hour_offset) = tz_hour {
        let minute_offset = tz_minute.unwrap_or(0);
        let total_seconds = (hour_offset as i32 * 3600) + (minute_offset as i32 * 60);
        let offset_seconds = if tz_minus {
            -total_seconds
        } else {
            total_seconds
        };

        if let Some(fixed_offset) = FixedOffset::east_opt(offset_seconds) {
            if let Some(dt_with_offset) = fixed_offset.from_local_datetime(&naive_dt).single() {
                return dt_with_offset.with_timezone(&Local);
            }
        }
    }

    // Try TZID parameter (IANA timezone name like "America/New_York")
    if let Some(tz_name) = tzid {
        if let Ok(iana_tz) = tz_name.parse::<IanaTz>() {
            if let Some(dt_with_tz) = iana_tz.from_local_datetime(&naive_dt).single() {
                return dt_with_tz.with_timezone(&Local);
            }
        }
    }

    // Floating time - assume it's already in local timezone
    Local
        .from_local_datetime(&naive_dt)
        .single()
        .unwrap_or_else(|| {
            // Fallback for ambiguous times (DST transitions)
            Local.from_local_datetime(&naive_dt).earliest().unwrap_or_else(|| {
                // Last resort - treat as UTC
                DateTime::from_naive_utc_and_offset(naive_dt, *Local::now().offset())
            })
        })
}

/// Get RRULE string from component
fn get_rrule_string(component: &ICalendarComponent) -> Option<String> {
    let entry = component.property(&ICalendarProperty::Rrule)?;
    entry.values.first().and_then(|v| {
        // RRULE is stored as RecurrenceRule variant
        if let ICalendarValue::RecurrenceRule(rule) = v {
            // Convert back to string for rrule crate
            Some(rule.to_string()) // This may need adjustment based on actual API
        } else if let ICalendarValue::Text(text) = v {
            Some(text.clone())
        } else {
            None
        }
    })
}

/// Expand RRULE to get occurrence dates within range
///
/// Safety limit: Returns at most 100 occurrences to prevent infinite expansion.
fn expand_rrule(
    rrule_str: &str,
    dtstart_date: NaiveDate,
    dtstart_time: Option<NaiveTime>,
    range_start: NaiveDate,
    range_end: NaiveDate,
) -> Result<Vec<NaiveDate>> {
    const MAX_OCCURRENCES: u16 = 100;

    // Build DTSTART string for rrule crate
    let dtstart_str = if let Some(time) = dtstart_time {
        format!(
            "DTSTART:{}T{}\n",
            dtstart_date.format("%Y%m%d"),
            time.format("%H%M%S")
        )
    } else {
        format!("DTSTART;VALUE=DATE:{}\n", dtstart_date.format("%Y%m%d"))
    };

    // Build full RRULE string
    let full_rrule = format!("{}{}", dtstart_str, ensure_rrule_prefix(rrule_str));

    // Parse with rrule crate
    let rrule_set: RRuleSet = full_rrule
        .parse()
        .map_err(|e| anyhow!("Failed to parse RRULE '{}': {:?}", rrule_str, e))?;

    // Set up range filter
    let after = Tz::UTC
        .with_ymd_and_hms(
            range_start.year(),
            range_start.month(),
            range_start.day(),
            0,
            0,
            0,
        )
        .single()
        .ok_or_else(|| anyhow!("Invalid range_start date"))?;

    let before = Tz::UTC
        .with_ymd_and_hms(
            range_end.year(),
            range_end.month(),
            range_end.day(),
            23,
            59,
            59,
        )
        .single()
        .ok_or_else(|| anyhow!("Invalid range_end date"))?;

    // Get occurrences within range
    let result = rrule_set.after(after).before(before).all(MAX_OCCURRENCES);

    // Extract dates
    let dates: Vec<NaiveDate> = result
        .dates
        .into_iter()
        .map(|dt| dt.date_naive())
        .collect();

    Ok(dates)
}

/// Ensure RRULE string has proper prefix
fn ensure_rrule_prefix(s: &str) -> String {
    if s.starts_with("RRULE:") {
        s.to_string()
    } else {
        format!("RRULE:{}", s)
    }
}

