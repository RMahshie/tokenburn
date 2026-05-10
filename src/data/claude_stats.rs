use std::collections::BTreeMap;

use color_eyre::eyre::Result;
use serde::Deserialize;

use crate::config;

use super::{ClaudeModelUsage, ClaudeUsageCache, MetricBreakdown};

pub fn load() -> Result<Option<ClaudeUsageCache>> {
    let path = config::claude_stats_cache_path()?;
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    let raw = serde_json::from_str::<RawStatsCache>(&contents)?;

    let mut total = MetricBreakdown::default();
    let mut models = Vec::new();
    for (name, usage) in raw.model_usage {
        let tokens = usage.into_breakdown();
        total.input += tokens.input;
        total.output += tokens.output;
        total.cache_read += tokens.cache_read;
        total.cache_write += tokens.cache_write;
        total.reasoning += tokens.reasoning;
        models.push(ClaudeModelUsage { name, tokens });
    }
    models.sort_by(|a, b| b.tokens.total().cmp(&a.tokens.total()));

    Ok(Some(ClaudeUsageCache {
        path,
        version: raw.version,
        first_session_date: raw.first_session_date,
        last_computed_date: raw.last_computed_date,
        total_messages: raw.total_messages,
        total_sessions: raw.total_sessions,
        total,
        models,
    }))
}

#[derive(Debug, Deserialize)]
struct RawStatsCache {
    version: Option<u64>,
    #[serde(rename = "firstSessionDate")]
    first_session_date: Option<String>,
    #[serde(rename = "lastComputedDate")]
    last_computed_date: Option<String>,
    #[serde(rename = "totalMessages")]
    total_messages: Option<u64>,
    #[serde(rename = "totalSessions")]
    total_sessions: Option<u64>,
    #[serde(rename = "modelUsage", default)]
    model_usage: BTreeMap<String, RawModelUsage>,
}

#[derive(Debug, Default, Deserialize)]
struct RawModelUsage {
    #[serde(rename = "inputTokens", default)]
    input_tokens: u64,
    #[serde(rename = "outputTokens", default)]
    output_tokens: u64,
    #[serde(rename = "cacheReadInputTokens", default)]
    cache_read_input_tokens: u64,
    #[serde(rename = "cacheCreationInputTokens", default)]
    cache_creation_input_tokens: u64,
}

impl RawModelUsage {
    fn into_breakdown(self) -> MetricBreakdown {
        MetricBreakdown {
            input: self.input_tokens,
            output: self.output_tokens,
            cache_read: self.cache_read_input_tokens,
            cache_write: self.cache_creation_input_tokens,
            reasoning: 0,
        }
    }
}
