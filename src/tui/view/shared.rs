use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::theme;

pub fn detail_line(label: &str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme::muted_style()),
        Span::raw(value.into()),
    ])
}

pub fn append_bottom_lines(
    lines: &mut Vec<Line<'static>>,
    bottom_lines: Vec<Line<'static>>,
    area: Rect,
    border_height: u16,
) {
    let content_height = area.height.saturating_sub(border_height) as usize;
    let blank_lines = content_height.saturating_sub(lines.len() + bottom_lines.len());
    lines.extend(std::iter::repeat_with(Line::default).take(blank_lines));
    lines.extend(bottom_lines);
}

pub fn render_card(frame: &mut Frame<'_>, area: Rect, title: &'static str, value: &str) {
    frame.render_widget(
        Paragraph::new(value.to_string())
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}
