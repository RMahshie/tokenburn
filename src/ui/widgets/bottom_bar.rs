use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    timestamp: &str,
    is_live: bool,
    claude_active: bool,
    can_apply_retention: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(64), Constraint::Length(46)])
        .split(area);

    let mut key_spans = vec![
        key("q"),
        Span::raw(" Quit   "),
        key("Tab"),
        Span::raw(" Provider   "),
    ];
    if claude_active {
        key_spans.push(key("s"));
        key_spans.push(Span::raw(" View   "));
    }
    if can_apply_retention {
        key_spans.push(key("a"));
        key_spans.push(Span::raw(" 10y   "));
    }
    key_spans.extend([
        key("r"),
        Span::raw(" Range   "),
        key("p"),
        Span::raw(" Pause   "),
        key("?"),
        Span::raw(" Help"),
    ]);
    let keys = Line::from(key_spans);
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
