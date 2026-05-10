use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, timestamp: &str, is_live: bool) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let keys = Line::from(vec![
        key("q"),
        Span::raw(" Quit   "),
        key("Tab"),
        Span::raw(" Provider   "),
        key("s"),
        Span::raw(" Subagents   "),
        key("r"),
        Span::raw(" Range   "),
        key("p"),
        Span::raw(" Pause   "),
        key("?"),
        Span::raw(" Help"),
    ]);
    frame.render_widget(
        Paragraph::new(keys).style(Style::new().fg(theme::MUTED)),
        chunks[0],
    );

    let mut right_spans = vec![Span::styled(
        format!("Data as of: {timestamp}  "),
        Style::new().fg(theme::MUTED),
    )];
    if is_live {
        right_spans.push(Span::styled("● Live", Style::new().fg(theme::SUCCESS)));
    }
    frame.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        chunks[1],
    );
}

fn key(value: &'static str) -> Span<'static> {
    Span::styled(format!(" {value} "), Style::new().fg(theme::TEXT).bold())
}
