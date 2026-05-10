use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{data::ChartBucket, ui::theme, util::format_compact};

pub fn render(frame: &mut Frame, area: Rect, buckets: &[ChartBucket], color: Color, range: &str) {
    let block = Block::default()
        .title(format!(" TOKEN BURN OVER TIME ({}) ", range.to_uppercase()))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::BORDER));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 12 || inner.height < 3 {
        return;
    }

    let label_width: u16 = 6;
    let plot_x = inner.x + label_width;
    let plot_width = inner.width.saturating_sub(label_width) as usize;
    let plot_height = inner.height.saturating_sub(1) as usize;

    if plot_width == 0 || plot_height == 0 {
        return;
    }

    let values: Vec<u64> = buckets
        .iter()
        .map(|bucket| bucket.breakdown.total())
        .collect();
    let n = values.len();
    let max = values.iter().copied().max().unwrap_or(0).max(1);
    let heights: Vec<usize> = values
        .iter()
        .map(|&value| {
            if value == 0 {
                0
            } else {
                ((value as f64 / max as f64) * plot_height as f64)
                    .ceil()
                    .max(1.0) as usize
            }
        })
        .collect();

    let bar_width = if n > 0 && n <= 30 && plot_width >= n * 3 {
        2
    } else {
        1
    };
    let bar_positions = bar_positions(n, bar_width, plot_width);

    for row in 0..plot_height {
        let mut spans: Vec<Span> = Vec::with_capacity(plot_width);

        let mut cursor = 0usize;
        for (i, &height) in heights.iter().enumerate() {
            let bar_x = bar_positions.get(i).copied().unwrap_or(cursor);
            if bar_x > cursor {
                spans.push(blank_span(bar_x - cursor));
            }
            let ch = if height > 0 && row >= plot_height.saturating_sub(height) {
                Span::styled("█".repeat(bar_width), Style::new().fg(color))
            } else if height == 0 && row == plot_height.saturating_sub(1) {
                zero_marker(bar_width)
            } else {
                blank_span(bar_width)
            };
            spans.push(ch);
            cursor = bar_x + bar_width;
        }

        if cursor < plot_width {
            spans.push(blank_span(plot_width - cursor));
        }

        let row_rect = Rect {
            x: plot_x,
            y: inner.y + row as u16,
            width: plot_width as u16,
            height: 1,
        };
        frame.render_widget(Paragraph::new(Line::from(spans)), row_rect);
    }

    let date_row = Rect {
        x: plot_x,
        y: inner.y + inner.height.saturating_sub(1),
        width: plot_width as u16,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            date_labels(buckets, plot_width, &bar_positions, bar_width),
            Style::new().fg(theme::MUTED),
        ))),
        date_row,
    );

    render_y_label(frame, inner.x, label_width, inner.y, &format_compact(max));
    if plot_height >= 4 {
        render_y_label(
            frame,
            inner.x,
            label_width,
            inner.y + (plot_height / 2) as u16,
            &format_compact(max / 2),
        );
    }
    render_y_label(
        frame,
        inner.x,
        label_width,
        inner.y + plot_height.saturating_sub(1) as u16,
        "0",
    );
}

fn render_y_label(frame: &mut Frame, x: u16, width: u16, y: u16, label: &str) {
    let rect = Rect {
        x,
        y,
        width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(label.to_string())
            .alignment(Alignment::Right)
            .style(Style::new().fg(theme::MUTED)),
        rect,
    );
}

fn blank_span(width: usize) -> Span<'static> {
    Span::raw(" ".repeat(width))
}

fn zero_marker(width: usize) -> Span<'static> {
    Span::styled("▁".repeat(width), Style::new().fg(theme::BORDER))
}

fn bar_positions(count: usize, bar_width: usize, plot_width: usize) -> Vec<usize> {
    if count == 0 || plot_width == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![plot_width.saturating_sub(bar_width) / 2];
    }

    let gap = usize::from(count * bar_width + count.saturating_sub(1) <= plot_width);
    let used_width = count * bar_width + count.saturating_sub(1) * gap;
    let offset = plot_width.saturating_sub(used_width) / 2;

    (0..count)
        .map(|idx| offset + idx * (bar_width + gap))
        .collect()
}

fn date_labels(
    buckets: &[ChartBucket],
    width: usize,
    bar_positions: &[usize],
    bar_width: usize,
) -> String {
    let Some(first) = buckets.first() else {
        return String::new();
    };
    let Some(last) = buckets.last() else {
        return String::new();
    };
    let left = first.start.format("%b %-d").to_string();
    let right = last.end.format("%b %-d").to_string();
    if width == 0 {
        return String::new();
    }

    let mut chars = vec![' '; width];
    let left_start = bar_positions.first().copied().unwrap_or(0).min(width);
    for (idx, ch) in left.chars().enumerate() {
        let pos = left_start + idx;
        if pos < width {
            chars[pos] = ch;
        }
    }

    let right_anchor = bar_positions
        .last()
        .copied()
        .unwrap_or(width.saturating_sub(right.len()))
        .saturating_add(bar_width)
        .min(width);
    let right_start = right_anchor.saturating_sub(right.len());
    for (idx, ch) in right.chars().enumerate() {
        let pos = right_start + idx;
        if pos < width {
            chars[pos] = ch;
        }
    }

    chars.into_iter().collect()
}
