//! iCal parsing with RRULE expansion

use anyhow::{anyhow, Result};
use calcard::icalendar::{
    ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarProperty, ICalendarValue,
};
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone};
use rrule::{RRuleSet, Tz};

use super::types::CalendarEvent;

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

/// Parse DTSTART from component, returns (date, time, is_all_day)
fn parse_dtstart(component: &ICalendarComponent) -> Option<(NaiveDate, Option<NaiveTime>, bool)> {
    let entry = component.property(&ICalendarProperty::Dtstart)?;
    entry.values.first().and_then(parse_datetime_value)
}

/// Parse DTEND from component
fn parse_dtend(component: &ICalendarComponent) -> Option<(Option<NaiveDate>, Option<NaiveTime>)> {
    let entry = component.property(&ICalendarProperty::Dtend)?;
    let (date, time, _) = entry.values.first().and_then(parse_datetime_value)?;
    Some((Some(date), time))
}

/// Parse a datetime value into (date, time, is_all_day)
fn parse_datetime_value(value: &ICalendarValue) -> Option<(NaiveDate, Option<NaiveTime>, bool)> {
    if let ICalendarValue::PartialDateTime(pdt) = value {
        // PartialDateTime has optional year, month, day fields
        let year = pdt.year?;
        let month = pdt.month?;
        let day = pdt.day?;
        let date = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)?;

        // Check if time is present (hour/minute/second)
        let (time, all_day) = if pdt.hour.is_some() {
            let t = NaiveTime::from_hms_opt(
                pdt.hour.unwrap_or(0) as u32,
                pdt.minute.unwrap_or(0) as u32,
                pdt.second.unwrap_or(0) as u32,
            )?;
            (Some(t), false)
        } else {
            (None, true)
        };

        Some((date, time, all_day))
    } else {
        None
    }
}

/// Get RRULE string from component
fn get_rrule_string(component: &ICalendarComponent) -> Option<String> {
    let entry = component.property(&ICalendarProperty::Rrule)?;
    entry.values.first().and_then(|v| {
        // RRULE is stored as RecurrenceRule variant
        if let ICalendarValue::RecurrenceRule(rule) = v {
            // Convert back to string for rrule crate
            Some(format!("{:?}", rule)) // This may need adjustment based on actual API
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

