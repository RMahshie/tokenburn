use ratatui::{
    style::Style,
    text::{Line, Span},
};

use crate::ui::theme;

pub fn line(change: Option<f64>) -> Line<'static> {
    match change {
        Some(value) if value >= 0.0 => Line::from(vec![
            Span::styled("↗ ", Style::new().fg(theme::SUCCESS)),
            Span::styled(format!("{value:.1}%"), Style::new().fg(theme::SUCCESS)),
            Span::styled(" vs prior period", Style::new().fg(theme::MUTED)),
        ]),
        Some(value) => Line::from(vec![
            Span::styled("↘ ", Style::new().fg(theme::ERROR)),
            Span::styled(
                format!("{:.1}%", value.abs()),
                Style::new().fg(theme::ERROR),
            ),
            Span::styled(" vs prior period", Style::new().fg(theme::MUTED)),
        ]),
        None => Line::from(Span::styled("no prior period", Style::new().fg(theme::DIM))),
    }
}
