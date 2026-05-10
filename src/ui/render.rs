use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{
    config,
    data::{DashboardData, MetricBreakdown, TimeRange, Tool},
    ui::theme,
    util::{format_tokens, local_timestamp},
};

use super::{
    app::ClaudeView,
    widgets::{bottom_bar, header, token_table},
};

pub const MIN_WIDTH: u16 = 110;
pub const MIN_HEIGHT: u16 = 34;

pub fn render(
    frame: &mut Frame,
    data: &DashboardData,
    range: &TimeRange,
    live: bool,
    paused: bool,
    show_help: bool,
    active_tab: Option<usize>,
    claude_view: ClaudeView,
) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, area);
        return;
    }
    let area = area.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });

    let section_height = area.height.saturating_sub(5 + 2 + 1).max(12);
    let constraints = vec![
        Constraint::Length(5),
        Constraint::Length(section_height),
        Constraint::Min(1),
        Constraint::Length(2),
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let refresh = if live {
        if paused {
            "paused"
        } else {
            "on"
        }
    } else {
        "off"
    };
    header::render(frame, chunks[0], range, refresh);

    if data.summaries.is_empty() {
        render_empty(frame, chunks[1]);
    } else {
        let tab = active_tab.unwrap_or(0).min(data.summaries.len() - 1);
        let base_summary = &data.summaries[tab];
        match (base_summary.tool, claude_view) {
            (Tool::Claude, ClaudeView::Subagents) => {
                if let Some(summary) = &data.claude_subagents {
                    token_table::render(frame, chunks[1], summary, range);
                } else {
                    render_empty_message(
                        frame,
                        chunks[1],
                        "No Claude subagent token data in this range.",
                    );
                }
            }
            (Tool::Claude, ClaudeView::Combined) => {
                if let Some(summary) = &data.claude_combined {
                    token_table::render(frame, chunks[1], summary, range);
                } else {
                    render_empty_message(frame, chunks[1], "No Claude token data in this range.");
                }
            }
            (Tool::Claude, ClaudeView::UsageCache) => {
                render_usage_cache(frame, chunks[1], data);
            }
            _ => token_table::render(frame, chunks[1], base_summary, range),
        }

        if data.summaries.len() > 1 {
            render_tabs(frame, chunks[0], data, tab, claude_view);
        }
    }

    let timestamp = local_timestamp(data.generated_at);
    let is_live = live && !paused;
    let active_tool = active_tab
        .and_then(|idx| data.summaries.get(idx))
        .map(|summary| summary.tool);
    let claude_active = active_tool == Some(Tool::Claude);
    let can_apply_retention = claude_active
        && data
            .claude_retention
            .as_ref()
            .is_some_and(|status| status.needs_update);
    bottom_bar::render(
        frame,
        chunks[chunks.len() - 1],
        &timestamp,
        is_live,
        claude_active,
        can_apply_retention,
    );

    if show_help {
        render_help(frame, centered_rect(58, 11, area));
    }
}

fn render_tabs(
    frame: &mut Frame,
    area: Rect,
    data: &DashboardData,
    active: usize,
    claude_view: ClaudeView,
) {
    let tab_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    let mut spans = Vec::new();
    for (i, summary) in data.summaries.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  │  ", Style::new().fg(theme::BORDER)));
        }
        let label = if summary.tool == Tool::Claude {
            format!(" {} ", claude_view.label())
        } else {
            format!(" {} ", summary.tool.label())
        };
        if i == active {
            spans.push(Span::styled(
                label,
                Style::new()
                    .fg(match summary.tool {
                        Tool::Claude | Tool::ClaudeSubagent | Tool::ClaudeAll => theme::CLAUDE,
                        Tool::Codex => theme::CODEX,
                    })
                    .bold(),
            ));
        } else {
            spans.push(Span::styled(label, Style::new().fg(theme::MUTED)));
        }
    }
    spans.push(Span::styled("  ← Tab →", Style::new().fg(theme::DIM)));
    frame.render_widget(Paragraph::new(Line::from(spans)), tab_area);
}

fn render_usage_cache(frame: &mut Frame, area: Rect, data: &DashboardData) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::BORDER));
    let inner = block.inner(area).inner(Margin {
        vertical: 1,
        horizontal: 3,
    });
    frame.render_widget(block, area);

    let Some(cache) = &data.claude_usage_cache else {
        render_empty_message(frame, area, "No Claude stats-cache.json file was found.");
        return;
    };

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(" CU ", Style::new().fg(theme::CLAUDE)),
        Span::raw("  "),
        Span::styled("CLAUDE USAGE CACHE", Style::new().fg(theme::CLAUDE).bold()),
    ]));
    lines.push(Line::raw(""));
    lines.push(info_line(
        "stats-cache total",
        format_tokens(cache.total.total()),
    ));
    lines.push(info_line(
        "available transcript total",
        format_tokens(data.claude_available_history.total()),
    ));
    lines.push(info_line(
        "difference",
        format_signed_difference(cache.total.total(), data.claude_available_history.total()),
    ));
    lines.push(Line::raw(""));
    lines.push(info_line(
        "first session",
        cache.first_session_date.as_deref().unwrap_or("unknown"),
    ));
    lines.push(info_line(
        "last computed",
        cache.last_computed_date.as_deref().unwrap_or("unknown"),
    ));
    lines.push(info_line(
        "messages / sessions",
        format!(
            "{} / {}",
            optional_number(cache.total_messages),
            optional_number(cache.total_sessions)
        ),
    ));
    lines.push(info_line("source", cache.path.display().to_string()));
    if let Some(version) = cache.version {
        lines.push(info_line("cache version", version.to_string()));
    }
    lines.push(Line::raw(""));
    lines.extend(retention_lines(data));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "MODEL BREAKDOWN",
        Style::new().fg(theme::MUTED).bold(),
    ));

    for model in cache.models.iter().take(6) {
        lines.push(model_line(&model.name, &model.tokens));
    }

    let paragraph = Paragraph::new(lines)
        .style(Style::new().fg(theme::MUTED))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn retention_lines(data: &DashboardData) -> Vec<Line<'static>> {
    let Some(status) = &data.claude_retention else {
        return vec![Line::styled(
            "Claude settings could not be inspected.",
            Style::new().fg(theme::WARNING),
        )];
    };

    if status.needs_update {
        vec![
            Line::from(vec![
                Span::styled(
                    "Transcript retention",
                    Style::new().fg(theme::WARNING).bold(),
                ),
                Span::styled(
                    format!(
                        ": currently {}, set to {} days for 10 years of local history.",
                        status
                            .cleanup_period_days
                            .map(|days| days.to_string())
                            .unwrap_or_else(|| "default 30 days".to_string()),
                        config::CLAUDE_RETENTION_DAYS
                    ),
                    Style::new().fg(theme::MUTED),
                ),
            ]),
            Line::from(vec![
                Span::styled(" a ", Style::new().fg(theme::TEXT).bold()),
                Span::styled(
                    "Add 10-year retention to Claude settings",
                    Style::new().fg(theme::CLAUDE).bold(),
                ),
                Span::styled(
                    "  command: tokenburn --fix-claude-retention",
                    Style::new().fg(theme::DIM),
                ),
            ]),
            Line::styled(
                "This keeps local Claude transcripts longer; they may contain prompts, tool output, and file excerpts.",
                Style::new().fg(theme::DIM),
            ),
        ]
    } else {
        vec![Line::from(vec![
            Span::styled(
                "Transcript retention",
                Style::new().fg(theme::CLAUDE).bold(),
            ),
            Span::styled(
                format!(
                    ": {} days in {}",
                    status
                        .cleanup_period_days
                        .unwrap_or(config::CLAUDE_RETENTION_DAYS),
                    status.path.display()
                ),
                Style::new().fg(theme::MUTED),
            ),
        ])]
    }
}

fn info_line(label: &'static str, value: impl ToString) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<28}"), Style::new().fg(theme::DIM)),
        Span::styled(value.to_string(), Style::new().fg(theme::TEXT)),
    ])
}

fn model_line(name: &str, tokens: &MetricBreakdown) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{name:<28}"), Style::new().fg(theme::DIM)),
        Span::styled(format_tokens(tokens.total()), Style::new().fg(theme::TEXT)),
        Span::styled(" total  ", Style::new().fg(theme::DIM)),
        Span::styled(format_tokens(tokens.input), Style::new().fg(theme::MUTED)),
        Span::styled(" in  ", Style::new().fg(theme::DIM)),
        Span::styled(
            format_tokens(tokens.cache_read),
            Style::new().fg(theme::MUTED),
        ),
        Span::styled(" read  ", Style::new().fg(theme::DIM)),
        Span::styled(
            format_tokens(tokens.cache_write),
            Style::new().fg(theme::MUTED),
        ),
        Span::styled(" write  ", Style::new().fg(theme::DIM)),
        Span::styled(format_tokens(tokens.output), Style::new().fg(theme::MUTED)),
        Span::styled(" out", Style::new().fg(theme::DIM)),
    ])
}

fn optional_number(value: Option<u64>) -> String {
    value
        .map(format_tokens)
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_signed_difference(left: u64, right: u64) -> String {
    if left >= right {
        format!("+{}", format_tokens(left - right))
    } else {
        format!("-{}", format_tokens(right - left))
    }
}

fn render_empty(frame: &mut Frame, area: Rect) {
    render_empty_message(
        frame,
        area,
        "tokenburn has no Claude Code or Codex token data in this range.",
    );
}

fn render_empty_message(frame: &mut Frame, area: Rect, message: &'static str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::BORDER));
    let paragraph = Paragraph::new(Line::from(message))
        .style(Style::new().fg(theme::MUTED))
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new(vec![
        Line::styled(config::APP_NAME, Style::new().fg(theme::CLAUDE).bold()),
        Line::raw("Terminal is too small for the dashboard."),
        Line::raw(format!(
            "Use at least {MIN_WIDTH} columns by {MIN_HEIGHT} rows."
        )),
    ])
    .style(Style::new().fg(theme::MUTED))
    .wrap(Wrap { trim: true });
    frame.render_widget(
        paragraph,
        area.inner(Margin {
            vertical: 2,
            horizontal: 2,
        }),
    );
}

fn render_help(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
    let text = vec![
        Line::styled("Help", Style::new().fg(theme::CLAUDE).bold()),
        Line::raw(""),
        Line::raw("Tab / ←→   Switch provider"),
        Line::raw("s          Cycle Claude views: main, subagents, all, usage cache"),
        Line::raw("a          Add Claude 10-year transcript retention when prompted"),
        Line::raw("r          Cycle range: 24h, 7d, 30d, lifetime"),
        Line::raw("p          Pause or resume live refresh"),
        Line::raw("?          Toggle this help"),
        Line::raw("q / Esc    Quit"),
    ];
    let widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" tokenburn "))
        .style(Style::new().fg(theme::MUTED));
    frame.render_widget(widget, area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Length(height.min(area.height)),
            Constraint::Percentage(50),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Length(width.min(area.width)),
            Constraint::Percentage(50),
        ])
        .split(vertical[1]);
    horizontal[1]
}
