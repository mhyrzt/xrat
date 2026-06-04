use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::theme;

pub fn render_key_bar(frame: &mut Frame<'_>, area: Rect) {
    let brand = Span::styled(
        concat!("XRAT v", env!("CARGO_PKG_VERSION")),
        theme::accent_style().bold(),
    );

    let actions = [
        Span::styled("actions:", theme::muted_style()),
        Span::raw(" "),
        Span::styled("[?]Help", theme::chrome_style()),
        Span::raw("   "),
        Span::styled("[q]uit", theme::chrome_style()),
    ];

    let total = area.width as usize;
    let brand_width = brand.width();
    let actions_width: usize = actions.iter().map(Span::width).sum();
    let gap = total
        .saturating_sub(brand_width + actions_width + theme::EDGE_MARGIN)
        .max(3);

    let mut spans = vec![brand, Span::raw(" ".repeat(gap))];
    spans.extend(actions);

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
