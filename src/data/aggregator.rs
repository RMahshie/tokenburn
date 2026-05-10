use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use color_eyre::eyre::Result;

use super::{
    claude, claude_stats, codex, ChartBucket, DashboardData, MetricBreakdown, TimeRange,
    TokenRecord, Tool, ToolSummary,
};

pub fn load_dashboard_data(range: &TimeRange) -> Result<DashboardData> {
    let mut records = claude::load()?;
    records.extend(codex::load()?);
    let mut data = aggregate(&records, range, Utc::now());
    data.claude_usage_cache = claude_stats::load()?;
    data.claude_retention = crate::config::claude_retention_status().ok();
    Ok(data)
}

pub fn aggregate(records: &[TokenRecord], range: &TimeRange, now: DateTime<Utc>) -> DashboardData {
    let mut summaries = Vec::new();
    for tool in [Tool::Claude, Tool::Codex] {
        if let Some(summary) = summarize_tool(records, tool, range, now) {
            summaries.push(summary);
        }
    }
    let claude_subagents = summarize_tool(records, Tool::ClaudeSubagent, range, now);
    let claude_combined = summarize_tools(
        records,
        &[Tool::Claude, Tool::ClaudeSubagent],
        Tool::ClaudeAll,
        range,
        now,
    );
    let claude_available_history =
        sum_available_history(records, &[Tool::Claude, Tool::ClaudeSubagent]);

    DashboardData {
        summaries,
        claude_subagents,
        claude_combined,
        claude_usage_cache: None,
        claude_available_history,
        claude_retention: None,
        generated_at: now,
    }
}

fn summarize_tool(
    records: &[TokenRecord],
    tool: Tool,
    range: &TimeRange,
    now: DateTime<Utc>,
) -> Option<ToolSummary> {
    summarize_tools(records, &[tool], tool, range, now)
}

fn summarize_tools(
    records: &[TokenRecord],
    tools: &[Tool],
    summary_tool: Tool,
    range: &TimeRange,
    now: DateTime<Utc>,
) -> Option<ToolSummary> {
    let tool_records: Vec<_> = records
        .iter()
        .filter(|record| tools.contains(&record.tool))
        .collect();
    if tool_records.is_empty() {
        return None;
    }

    let (start, end) = range_bounds(range, now, &tool_records);
    let period = end - start;
    let previous_start = start - period;
    let previous_end = start;

    let mut current = MetricBreakdown::default();
    let mut previous = MetricBreakdown::default();
    let mut by_day: BTreeMap<_, MetricBreakdown> = BTreeMap::new();

    for record in &tool_records {
        if record.timestamp >= start && record.timestamp <= end {
            current.add_record(record);
            by_day
                .entry(record.timestamp.date_naive())
                .or_default()
                .add_record(record);
        } else if record.timestamp >= previous_start && record.timestamp < previous_end {
            previous.add_record(record);
        }
    }

    let day_count = match range {
        TimeRange::LastHours(_) => 1,
        TimeRange::LastDays(days) => (*days).max(1) as u64,
        TimeRange::Custom { start, end } => {
            (end.date_naive() - start.date_naive()).num_days().max(0) as u64 + 1
        }
        TimeRange::Lifetime => by_day.len().max(1) as u64,
    };

    Some(ToolSummary {
        tool: summary_tool,
        daily_average: divide_breakdown(&current, day_count),
        percent_change: percent_change(current.total(), previous.total()),
        chart_buckets: build_chart_buckets(&tool_records, start, end, range),
        current,
    })
}

fn sum_available_history(records: &[TokenRecord], tools: &[Tool]) -> MetricBreakdown {
    let mut total = MetricBreakdown::default();
    for record in records {
        if tools.contains(&record.tool) {
            total.add_record(record);
        }
    }
    total
}

fn range_bounds(
    range: &TimeRange,
    now: DateTime<Utc>,
    records: &[&TokenRecord],
) -> (DateTime<Utc>, DateTime<Utc>) {
    match range {
        TimeRange::LastHours(hours) => (now - Duration::hours(*hours), now),
        TimeRange::LastDays(days) => (now - Duration::days(*days), now),
        TimeRange::Custom { start, end } => (*start, *end),
        TimeRange::Lifetime => {
            let start = records
                .iter()
                .map(|record| record.timestamp)
                .min()
                .unwrap_or(now);
            (start, now)
        }
    }
}

fn build_chart_buckets(
    records: &[&TokenRecord],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    range: &TimeRange,
) -> Vec<ChartBucket> {
    let count = chart_bucket_count(range).max(1);
    let total_ms = (end - start).num_milliseconds().max(1);
    let mut buckets: Vec<ChartBucket> = (0..count)
        .map(|idx| {
            let bucket_start = start + Duration::milliseconds(total_ms * idx as i64 / count as i64);
            let bucket_end =
                start + Duration::milliseconds(total_ms * (idx + 1) as i64 / count as i64);
            ChartBucket {
                start: bucket_start,
                end: bucket_end,
                breakdown: MetricBreakdown::default(),
            }
        })
        .collect();

    for record in records {
        if record.timestamp < start || record.timestamp > end {
            continue;
        }
        let elapsed_ms = (record.timestamp - start)
            .num_milliseconds()
            .clamp(0, total_ms);
        let idx = ((elapsed_ms as i128 * count as i128) / total_ms as i128).min((count - 1) as i128)
            as usize;
        buckets[idx].breakdown.add_record(record);
    }

    buckets
}

fn chart_bucket_count(range: &TimeRange) -> usize {
    match range {
        TimeRange::LastDays(7) => 7,
        TimeRange::LastHours(_) | TimeRange::LastDays(_) | TimeRange::Lifetime => 72,
        TimeRange::Custom { .. } => 72,
    }
}

fn divide_breakdown(value: &MetricBreakdown, divisor: u64) -> MetricBreakdown {
    let divisor = divisor.max(1);
    MetricBreakdown {
        input: value.input / divisor,
        output: value.output / divisor,
        cache_read: value.cache_read / divisor,
        cache_write: value.cache_write / divisor,
        reasoning: value.reasoning / divisor,
    }
}

fn percent_change(current: u64, previous: u64) -> Option<f64> {
    if previous == 0 {
        return None;
    }
    Some(((current as f64 - previous as f64) / previous as f64) * 100.0)
}
