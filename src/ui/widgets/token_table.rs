use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    data::{MetricBreakdown, TimeRange, Tool, ToolSummary},
    ui::theme,
    util::{format_tokens, percent, range_label},
};

use super::{change_badge, line_chart};

pub fn render(frame: &mut Frame, area: Rect, summary: &ToolSummary, range: &TimeRange) {
    let color = tool_color(summary.tool);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::BORDER));
    let inner = block.inner(area).inner(Margin {
        vertical: 0,
        horizontal: 2,
    });
    frame.render_widget(block, area);

    let chart_height = inner.height.saturating_sub(8).max(3);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(chart_height),
        ])
        .split(inner);

    render_title(frame, chunks[0], summary, color);
    render_table(frame, chunks[1], summary, range, color);
    line_chart::render(
        frame,
        chunks[2],
        &summary.chart_buckets,
        color,
        &range_label(range),
    );
}

fn render_title(frame: &mut Frame, area: Rect, summary: &ToolSummary, color: Color) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let title = Line::from(vec![
        Span::styled(
            format!(" {} ", summary.tool.short()),
            Style::new().fg(color),
        ),
        Span::raw("  "),
        Span::styled(summary.tool.label(), Style::new().fg(color).bold()),
    ]);
    frame.render_widget(
        Paragraph::new(title),
        Rect {
            height: 1,
            ..chunks[0]
        },
    );
    frame.render_widget(
        Paragraph::new(change_badge::line(summary.percent_change)).alignment(Alignment::Right),
        Rect {
            height: 1,
            ..chunks[1]
        },
    );
}

fn render_table(
    frame: &mut Frame,
    area: Rect,
    summary: &ToolSummary,
    range: &TimeRange,
    color: Color,
) {
    let total = summary.current.total();
    let rows = metric_rows(summary);
    let widths = [16u16, 18, 16, 10, 14, 30];
    let mut x_positions = [0u16; 6];
    for idx in 1..x_positions.len() {
        x_positions[idx] = x_positions[idx - 1] + widths[idx - 1] + 1;
    }

    frame.render_widget(Clear, area);

    let header_style = Style::new().fg(theme::MUTED).add_modifier(Modifier::BOLD);
    let headers = [
        "METRIC".to_string(),
        format!("{} TOTAL", range_label(range).to_uppercase()),
        "DAILY AVG".to_string(),
        "% TOTAL".to_string(),
        String::new(),
        "DAILY TREND".to_string(),
    ];
    for (idx, header) in headers.into_iter().enumerate() {
        render_cell(
            frame,
            area,
            x_positions[idx],
            0,
            widths[idx],
            Line::from(Span::styled(header, header_style)),
        );
    }

    for (row_idx, row) in rows.into_iter().enumerate() {
        let y = row_idx as u16 + 1;
        if y >= area.height {
            break;
        }
        let style = Style::new().fg(color);
        let trend_values: Vec<u64> = summary
            .chart_buckets
            .iter()
            .map(|bucket| row.metric.value(&bucket.breakdown))
            .collect();
        let trend_values = compact_zero_runs(&trend_values, 2);
        let trend_width = trend_values.len().min(30);

        render_cell(
            frame,
            area,
            x_positions[0],
            y,
            widths[0],
            Line::from(Span::styled(row.label, style)),
        );
        render_cell(
            frame,
            area,
            x_positions[1],
            y,
            widths[1],
            Line::from(Span::styled(format_tokens(row.value), style)),
        );
        render_cell(
            frame,
            area,
            x_positions[2],
            y,
            widths[2],
            Line::from(Span::styled(format_tokens(row.avg), style)),
        );
        render_cell(
            frame,
            area,
            x_positions[3],
            y,
            widths[3],
            Line::from(Span::styled(
                format!("{:.1}%", percent(row.value, total)),
                style,
            )),
        );
        render_cell(
            frame,
            area,
            x_positions[4],
            y,
            widths[4],
            bar(percent(row.value, total), color),
        );
        render_cell(
            frame,
            area,
            x_positions[5],
            y,
            widths[5],
            Line::from(Span::styled(spark_trend(&trend_values, trend_width), style)),
        );
    }
}

fn render_cell(frame: &mut Frame, area: Rect, x: u16, y: u16, width: u16, line: Line<'static>) {
    if y >= area.height || x >= area.width {
        return;
    }
    let rect = Rect {
        x: area.x + x,
        y: area.y + y,
        width: width.min(area.width - x),
        height: 1,
    };
    frame.render_widget(Paragraph::new(line), rect);
}

fn metric_rows(summary: &ToolSummary) -> Vec<MetricRow> {
    let current = &summary.current;
    let average = &summary.daily_average;
    let mut metrics = vec![
        MetricRow::new("Input", current.input, average.input, MetricKind::Input),
        MetricRow::new("Output", current.output, average.output, MetricKind::Output),
        MetricRow::new(
            if summary.tool == Tool::Codex {
                "Cached Input"
            } else {
                "Cache Read"
            },
            current.cache_read,
            average.cache_read,
            MetricKind::CacheRead,
        ),
    ];
    if current.cache_write > 0 || summary.tool.is_claude() {
        metrics.push(MetricRow::new(
            "Cache Write",
            current.cache_write,
            average.cache_write,
            MetricKind::CacheWrite,
        ));
    }
    if current.reasoning > 0 || summary.tool == Tool::Codex {
        metrics.push(MetricRow::new(
            "Reasoning",
            current.reasoning,
            average.reasoning,
            MetricKind::Reasoning,
        ));
    }
    metrics.push(MetricRow::new(
        "Total",
        current.total(),
        average.total(),
        MetricKind::Total,
    ));
    metrics
}

struct MetricRow {
    label: &'static str,
    value: u64,
    avg: u64,
    metric: MetricKind,
}

impl MetricRow {
    fn new(label: &'static str, value: u64, avg: u64, metric: MetricKind) -> Self {
        Self {
            label,
            value,
            avg,
            metric,
        }
    }
}

fn bar(percent: f64, color: Color) -> Line<'static> {
    let width = 12;
    let scaled = ((percent.clamp(0.0, 100.0) / 100.0) * width as f64 * 8.0).round() as usize;
    let full = (scaled / 8).min(width);
    let partial = scaled % 8;
    let empty = width.saturating_sub(full + usize::from(partial > 0));
    let mut value = "█".repeat(full);
    if partial > 0 && full < width {
        value.push(PARTIAL_BLOCKS[partial]);
    }
    Line::from(vec![
        Span::styled(value, Style::new().fg(color)),
        Span::raw(" ".repeat(empty)),
    ])
}

const PARTIAL_BLOCKS: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
const BARS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn spark_trend(values: &[u64], width: usize) -> String {
    if width == 0 || values.is_empty() {
        return String::new();
    }

    let sampled = sample(values, width);
    let max = sampled.iter().copied().max().unwrap_or(0);
    if max == 0 {
        return " ".repeat(width);
    }

    sampled
        .iter()
        .map(|v| {
            let level = ((*v as f64 / max as f64) * 8.0).round() as usize;
            BARS[level.min(8)]
        })
        .collect()
}

fn compact_zero_runs(values: &[u64], max_zero_run: usize) -> Vec<u64> {
    let mut compacted = Vec::with_capacity(values.len());
    let mut zero_run = 0usize;

    for &value in values {
        if value == 0 {
            if zero_run < max_zero_run {
                compacted.push(0);
            }
            zero_run += 1;
        } else {
            compacted.push(value);
            zero_run = 0;
        }
    }

    compacted
}

fn sample(values: &[u64], width: usize) -> Vec<u64> {
    if values.is_empty() || width == 0 {
        return Vec::new();
    }
    if values.len() == 1 {
        return vec![values[0]; width];
    }
    if values.len() <= width {
        return (0..width)
            .map(|idx| {
                let source = (idx * (values.len() - 1)) / (width - 1).max(1);
                values[source]
            })
            .collect();
    }

    (0..width)
        .map(|idx| {
            let start = idx * values.len() / width;
            let end = ((idx + 1) * values.len() / width).max(start + 1);
            values[start..end].iter().copied().max().unwrap_or(0)
        })
        .collect()
}

#[derive(Clone, Copy)]
enum MetricKind {
    Input,
    Output,
    CacheRead,
    CacheWrite,
    Reasoning,
    Total,
}

impl MetricKind {
    fn value(self, breakdown: &MetricBreakdown) -> u64 {
        match self {
            MetricKind::Input => breakdown.input,
            MetricKind::Output => breakdown.output,
            MetricKind::CacheRead => breakdown.cache_read,
            MetricKind::CacheWrite => breakdown.cache_write,
            MetricKind::Reasoning => breakdown.reasoning,
            MetricKind::Total => breakdown.total(),
        }
    }
}

fn tool_color(tool: Tool) -> Color {
    match tool {
        Tool::Claude | Tool::ClaudeSubagent => theme::CLAUDE,
        Tool::Codex => theme::CODEX,
    }
}
