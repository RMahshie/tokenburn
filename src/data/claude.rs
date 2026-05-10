use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use glob::glob;
use rayon::prelude::*;
use serde_json::Value;

use crate::config;

use super::{TokenRecord, Tool};

pub fn load() -> Result<Vec<TokenRecord>> {
    let pattern = config::claude_glob()?;
    let paths: Vec<_> = glob(&pattern)?.filter_map(|entry| entry.ok()).collect();

    let records: Vec<_> = paths
        .par_iter()
        .flat_map(|path| parse_file(path).unwrap_or_default())
        .collect();

    Ok(deduplicate_requests(records))
}

fn parse_file(path: &Path) -> Result<Vec<ClaudeUsageRecord>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (idx, line) in reader.lines().map_while(|line| line.ok()).enumerate() {
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(usage) = value.pointer("/message/usage") else {
            continue;
        };
        let Some(timestamp) = value
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
            .map(|dt| dt.with_timezone(&Utc))
        else {
            continue;
        };
        let request_id = request_id(&value, path, idx);
        let tool = if is_subagent(&value, path) {
            Tool::ClaudeSubagent
        } else {
            Tool::Claude
        };

        records.push(ClaudeUsageRecord {
            request_id,
            record: TokenRecord {
                timestamp,
                tool,
                input: read_u64(usage, "input_tokens"),
                output: read_u64(usage, "output_tokens"),
                cache_write: read_u64(usage, "cache_creation_input_tokens"),
                cache_read: read_u64(usage, "cache_read_input_tokens"),
                reasoning: 0,
            },
        });
    }

    Ok(records)
}

fn request_id(value: &Value, path: &Path, line_idx: usize) -> String {
    value
        .get("requestId")
        .or_else(|| value.get("uuid"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{}:{line_idx}", path.display()))
}

fn is_subagent(value: &Value, path: &Path) -> bool {
    value
        .get("isSidechain")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || path
            .components()
            .any(|part| part.as_os_str().to_string_lossy() == "subagents")
}

fn deduplicate_requests(records: Vec<ClaudeUsageRecord>) -> Vec<TokenRecord> {
    let mut latest_by_request = HashMap::<String, TokenRecord>::new();

    for usage in records {
        latest_by_request
            .entry(usage.request_id)
            .and_modify(|existing| {
                if usage.record.timestamp >= existing.timestamp {
                    *existing = usage.record.clone();
                }
            })
            .or_insert(usage.record);
    }

    latest_by_request.into_values().collect()
}

fn read_u64(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

#[derive(Debug)]
struct ClaudeUsageRecord {
    request_id: String,
    record: TokenRecord,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(request_id: &str, timestamp: &str, output: u64) -> ClaudeUsageRecord {
        ClaudeUsageRecord {
            request_id: request_id.to_string(),
            record: TokenRecord {
                timestamp: DateTime::parse_from_rfc3339(timestamp)
                    .expect("valid timestamp")
                    .with_timezone(&Utc),
                tool: Tool::Claude,
                input: 10,
                output,
                cache_read: 100,
                cache_write: 20,
                reasoning: 0,
            },
        }
    }

    #[test]
    fn claude_usage_keeps_latest_record_for_each_request_id() {
        let records = deduplicate_requests(vec![
            record("req_a", "2026-05-10T00:00:00Z", 1),
            record("req_a", "2026-05-10T00:00:02Z", 9),
            record("req_b", "2026-05-10T00:00:01Z", 3),
        ]);

        assert_eq!(records.len(), 2);
        assert!(records.iter().any(|record| record.output == 9));
        assert!(records.iter().any(|record| record.output == 3));
        assert!(!records.iter().any(|record| record.output == 1));
    }
}
