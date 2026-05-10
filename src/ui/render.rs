use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{
    config,
    data::{DashboardData, TimeRange, Tool},
    ui::theme,
    util::local_timestamp,
};

use super::widgets::{bottom_bar, header, token_table};

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
    show_claude_subagents: bool,
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
        if show_claude_subagents && base_summary.tool == Tool::Claude {
            if let Some(summary) = &data.claude_subagents {
                token_table::render(frame, chunks[1], summary, range);
            } else {
                render_empty_message(
                    frame,
                    chunks[1],
                    "No Claude subagent token data in this range.",
                );
            }
        } else {
            token_table::render(frame, chunks[1], base_summary, range);
        }

        if data.summaries.len() > 1 {
            render_tabs(frame, chunks[0], data, tab, show_claude_subagents);
        }
    }

    let timestamp = local_timestamp(data.generated_at);
    let is_live = live && !paused;
    bottom_bar::render(frame, chunks[chunks.len() - 1], &timestamp, is_live);

    if show_help {
        render_help(frame, centered_rect(58, 11, area));
    }
}

fn render_tabs(
    frame: &mut Frame,
    area: Rect,
    data: &DashboardData,
    active: usize,
    show_claude_subagents: bool,
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
        let label = if summary.tool == Tool::Claude && show_claude_subagents {
            " CLAUDE SUBAGENTS ".to_string()
        } else {
            format!(" {} ", summary.tool.label())
        };
        if i == active {
            spans.push(Span::styled(
                label,
                Style::new()
                    .fg(match summary.tool {
                        Tool::Claude | Tool::ClaudeSubagent => theme::CLAUDE,
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
        Line::raw("s          Toggle Claude main/subagent usage"),
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
