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

pub fn render_card(frame: &mut Frame<'_>, area: Rect, title: &'static str, value: &str) {
    frame.render_widget(
        Paragraph::new(value.to_string())
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}
