use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use color_eyre::eyre::Result;
use glob::glob;
use rayon::prelude::*;
use serde_json::Value;

use crate::config;

use super::{TokenRecord, Tool};

pub fn load() -> Result<Vec<TokenRecord>> {
    let pattern = config::codex_glob()?;
    let paths: Vec<_> = glob(&pattern)?.filter_map(|entry| entry.ok()).collect();

    let records = paths
        .par_iter()
        .flat_map(|path| parse_file(path).unwrap_or_default())
        .collect();

    Ok(records)
}

fn parse_file(path: &Path) -> Result<Vec<TokenRecord>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut previous: Option<TokenUsage> = None;

    for line in reader.lines().map_while(|line| line.ok()) {
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("event_msg")
            || value.pointer("/payload/type").and_then(Value::as_str) != Some("token_count")
        {
            continue;
        }

        let timestamp = value
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
            .map(|dt| dt.with_timezone(&Utc));
        if let Some(usage) = value.pointer("/payload/info/total_token_usage") {
            let current = TokenUsage::from_value(usage);
            let delta = match previous {
                Some(prev) if current.is_at_least(prev) => current - prev,
                Some(_) | None => current,
            };
            previous = Some(current);
            if delta.is_zero() {
                continue;
            }
            records.push(delta.into_record(timestamp.unwrap_or_else(|| timestamp_from_path(path))));
        }
    }

    Ok(records)
}

fn timestamp_from_path(path: &Path) -> DateTime<Utc> {
    let parts: Vec<_> = path
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect();
    let mut date = None;
    for window in parts.windows(4) {
        if window[0] == "sessions" {
            let parsed = (
                window[1].parse::<i32>(),
                window[2].parse::<u32>(),
                window[3].parse::<u32>(),
            );
            if let (Ok(year), Ok(month), Ok(day)) = parsed {
                date = NaiveDate::from_ymd_opt(year, month, day);
            }
        }
    }

    Utc.from_utc_datetime(
        &date
            .unwrap_or_else(|| Utc::now().date_naive())
            .and_hms_opt(12, 0, 0)
            .expect("valid noon"),
    )
}

fn read_u64(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenUsage {
    input: u64,
    cached_input: u64,
    output: u64,
    reasoning_output: u64,
}

impl TokenUsage {
    fn from_value(value: &Value) -> Self {
        Self {
            input: read_u64(value, "input_tokens"),
            cached_input: read_u64(value, "cached_input_tokens"),
            output: read_u64(value, "output_tokens"),
            reasoning_output: read_u64(value, "reasoning_output_tokens"),
        }
    }

    fn is_zero(self) -> bool {
        self.input == 0 && self.cached_input == 0 && self.output == 0 && self.reasoning_output == 0
    }

    fn is_at_least(self, other: Self) -> bool {
        self.input >= other.input
            && self.cached_input >= other.cached_input
            && self.output >= other.output
            && self.reasoning_output >= other.reasoning_output
    }

    fn into_record(self, timestamp: DateTime<Utc>) -> TokenRecord {
        TokenRecord {
            timestamp,
            tool: Tool::Codex,
            input: self.input.saturating_sub(self.cached_input),
            output: self.output.saturating_sub(self.reasoning_output),
            cache_read: self.cached_input,
            cache_write: 0,
            reasoning: self.reasoning_output,
        }
    }
}

impl std::ops::Sub for TokenUsage {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            input: self.input.saturating_sub(rhs.input),
            cached_input: self.cached_input.saturating_sub(rhs.cached_input),
            output: self.output.saturating_sub(rhs.output),
            reasoning_output: self.reasoning_output.saturating_sub(rhs.reasoning_output),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_usage_splits_cached_input_and_reasoning_out_of_totals() {
        let usage = TokenUsage {
            input: 1_000,
            cached_input: 800,
            output: 100,
            reasoning_output: 25,
        };

        let record = usage.into_record(Utc::now());

        assert_eq!(record.input, 200);
        assert_eq!(record.cache_read, 800);
        assert_eq!(record.output, 75);
        assert_eq!(record.reasoning, 25);
        assert_eq!(
            record.input + record.cache_read + record.output + record.reasoning,
            1_100
        );
    }

    #[test]
    fn codex_usage_delta_uses_cumulative_total_difference() {
        let previous = TokenUsage {
            input: 1_000,
            cached_input: 800,
            output: 100,
            reasoning_output: 25,
        };
        let current = TokenUsage {
            input: 1_500,
            cached_input: 1_100,
            output: 150,
            reasoning_output: 35,
        };

        let delta = current - previous;

        assert_eq!(
            delta,
            TokenUsage {
                input: 500,
                cached_input: 300,
                output: 50,
                reasoning_output: 10,
            }
        );
    }
}
