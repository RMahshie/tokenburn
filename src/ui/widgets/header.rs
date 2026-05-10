use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{config, data::TimeRange, ui::theme, util::range_label};

pub fn render(frame: &mut Frame, area: Rect, range: &TimeRange, refresh: &str) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[1]);

    let title = Line::from(vec![
        Span::styled(config::APP_NAME, Style::new().fg(theme::CLAUDE).bold()),
        Span::raw("  "),
        Span::styled(
            format!("v{}", config::VERSION),
            Style::new().fg(theme::MUTED),
        ),
    ]);
    frame.render_widget(Paragraph::new(title), top[0]);

    let right = Line::from(vec![
        Span::raw("Time range: "),
        Span::styled(range_label(range), Style::new().fg(theme::TEXT)),
        Span::raw("   Auto-refresh: "),
        Span::styled(
            refresh,
            Style::new().fg(if refresh == "on" {
                theme::SUCCESS
            } else {
                theme::WARNING
            }),
        ),
    ]);
    frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), top[1]);

    frame.render_widget(
        Paragraph::new("Track token usage and burn for Claude Code and Codex")
            .style(Style::new().fg(theme::MUTED)),
        rows[2],
    );
    frame.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(theme::BORDER)),
        rows[3],
    );
}
