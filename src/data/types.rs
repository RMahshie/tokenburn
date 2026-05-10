use chrono::{DateTime, Utc};

use crate::config::ClaudeRetentionStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Claude,
    ClaudeSubagent,
    ClaudeAll,
    Codex,
}

impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Tool::Claude => "CLAUDE CODE",
            Tool::ClaudeSubagent => "CLAUDE SUBAGENTS",
            Tool::ClaudeAll => "CLAUDE ALL",
            Tool::Codex => "CODEX",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Tool::Claude => "CC",
            Tool::ClaudeSubagent => "CS",
            Tool::ClaudeAll => "CA",
            Tool::Codex => "CX",
        }
    }

    pub fn is_claude(self) -> bool {
        matches!(self, Tool::Claude | Tool::ClaudeSubagent | Tool::ClaudeAll)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MetricBreakdown {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub reasoning: u64,
}

impl MetricBreakdown {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write + self.reasoning
    }

    pub fn add_record(&mut self, record: &TokenRecord) {
        self.input += record.input;
        self.output += record.output;
        self.cache_read += record.cache_read;
        self.cache_write += record.cache_write;
        self.reasoning += record.reasoning;
    }
}

#[derive(Debug, Clone)]
pub struct TokenRecord {
    pub timestamp: DateTime<Utc>,
    pub tool: Tool,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub reasoning: u64,
}

#[derive(Debug, Clone)]
pub struct ChartBucket {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub breakdown: MetricBreakdown,
}

#[derive(Debug, Clone)]
pub struct ToolSummary {
    pub tool: Tool,
    pub current: MetricBreakdown,
    pub daily_average: MetricBreakdown,
    pub percent_change: Option<f64>,
    pub chart_buckets: Vec<ChartBucket>,
}

#[derive(Debug, Clone)]
pub struct DashboardData {
    pub summaries: Vec<ToolSummary>,
    pub claude_subagents: Option<ToolSummary>,
    pub claude_combined: Option<ToolSummary>,
    pub claude_usage_cache: Option<ClaudeUsageCache>,
    pub claude_available_history: MetricBreakdown,
    pub claude_retention: Option<ClaudeRetentionStatus>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeRange {
    LastHours(i64),
    LastDays(i64),
    Lifetime,
    Custom {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

#[derive(Debug, Clone)]
pub struct ClaudeUsageCache {
    pub path: std::path::PathBuf,
    pub version: Option<u64>,
    pub first_session_date: Option<String>,
    pub last_computed_date: Option<String>,
    pub total_messages: Option<u64>,
    pub total_sessions: Option<u64>,
    pub total: MetricBreakdown,
    pub models: Vec<ClaudeModelUsage>,
}

#[derive(Debug, Clone)]
pub struct ClaudeModelUsage {
    pub name: String,
    pub tokens: MetricBreakdown,
}
