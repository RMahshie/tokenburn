use chrono::{DateTime, Local, Utc};

use crate::data::TimeRange;

pub fn format_tokens(value: u64) -> String {
    let raw = value.to_string();
    let mut out = String::new();
    for (index, ch) in raw.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

pub fn format_compact(value: u64) -> String {
    if value >= 1_000_000_000 {
        format!("{:.1}B", value as f64 / 1_000_000_000.0)
    } else if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

pub fn percent(value: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        value as f64 / total as f64 * 100.0
    }
}

pub fn range_label(range: &TimeRange) -> String {
    match range {
        TimeRange::LastHours(24) => "24h".to_string(),
        TimeRange::LastHours(hours) => format!("{hours}h"),
        TimeRange::LastDays(7) => "7d".to_string(),
        TimeRange::LastDays(30) => "30d".to_string(),
        TimeRange::LastDays(days) => format!("{days}d"),
        TimeRange::Lifetime => "lifetime".to_string(),
        TimeRange::Custom { start, end } => {
            format!("{} - {}", start.date_naive(), end.date_naive())
        }
    }
}

pub fn local_timestamp(value: DateTime<Utc>) -> String {
    value
        .with_timezone(&Local)
        .format("%b %-d, %Y %H:%M:%S")
        .to_string()
}
