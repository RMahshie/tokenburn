use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    Claude,
    ClaudeSubagent,
    Codex,
}

impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Tool::Claude => "CLAUDE CODE",
            Tool::ClaudeSubagent => "CLAUDE SUBAGENTS",
            Tool::Codex => "CODEX",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Tool::Claude => "CC",
            Tool::ClaudeSubagent => "CS",
            Tool::Codex => "CX",
        }
    }

    pub fn is_claude(self) -> bool {
        matches!(self, Tool::Claude | Tool::ClaudeSubagent)
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
