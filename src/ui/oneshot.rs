use std::io::{self, IsTerminal, Write};

use color_eyre::eyre::Result;
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};

use crate::{
    data::{DashboardData, MetricBreakdown, TimeRange, ToolSummary},
    util::{format_tokens, local_timestamp, percent, range_label},
};

use super::{app::ClaudeView, render};

pub fn run(data: DashboardData, range: TimeRange) -> Result<()> {
    if !io::stdout().is_terminal() {
        return render_plain(&data, &range);
    }

    let backend = CrosstermBackend::new(io::stdout());
    let options = TerminalOptions {
        viewport: Viewport::Inline(44),
    };
    let mut terminal = Terminal::with_options(backend, options)?;
    terminal.draw(|frame| {
        render::render(
            frame,
            &data,
            &range,
            false,
            false,
            false,
            None,
            ClaudeView::Main,
            false,
        )
    })?;
    Ok(())
}

fn render_plain(data: &DashboardData, range: &TimeRange) -> Result<()> {
    let mut out = io::stdout();
    writeln!(out, "tokenburn v{}", env!("CARGO_PKG_VERSION"))?;
    writeln!(out, "Track token usage and burn for Claude Code and Codex")?;
    writeln!(out, "Time range: {}", range_label(range))?;
    writeln!(out, "Data as of: {}", local_timestamp(data.generated_at))?;

    if data.summaries.is_empty() {
        writeln!(out, "\nNo Claude Code or Codex token data in this range.")?;
        return Ok(());
    }

    for summary in &data.summaries {
        render_plain_summary(&mut out, summary, range)?;
    }

    Ok(())
}

fn render_plain_summary(
    out: &mut impl Write,
    summary: &ToolSummary,
    range: &TimeRange,
) -> Result<()> {
    writeln!(out, "\n{}", summary.tool.label())?;
    match summary.percent_change {
        Some(change) => writeln!(out, "Change vs prior period: {change:.1}%")?,
        None => writeln!(out, "Change vs prior period: n/a")?,
    }
    writeln!(
        out,
        "{:<14} {:>16} {:>16} {:>9}",
        "Metric",
        format!("{} total", range_label(range)),
        "Daily avg",
        "% total"
    )?;

    for (label, total, average) in metric_rows(summary) {
        writeln!(
            out,
            "{:<14} {:>16} {:>16} {:>8.1}%",
            label,
            format_tokens(total),
            format_tokens(average),
            percent(total, summary.current.total())
        )?;
    }

    Ok(())
}

fn metric_rows(summary: &ToolSummary) -> Vec<(&'static str, u64, u64)> {
    let MetricBreakdown {
        input,
        output,
        cache_read,
        cache_write,
        reasoning,
    } = summary.current;
    let avg = &summary.daily_average;
    let mut rows = vec![
        ("Input", input, avg.input),
        ("Output", output, avg.output),
        (
            if summary.tool == crate::data::Tool::Codex {
                "Cached Input"
            } else {
                "Cache Read"
            },
            cache_read,
            avg.cache_read,
        ),
    ];
    if cache_write > 0 {
        rows.push(("Cache Write", cache_write, avg.cache_write));
    }
    if reasoning > 0 {
        rows.push(("Reasoning", reasoning, avg.reasoning));
    }
    rows.push(("Total", summary.current.total(), avg.total()));
    rows
}
