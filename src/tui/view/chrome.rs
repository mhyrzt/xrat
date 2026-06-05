use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render_key_bar(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let mut brand = vec![Span::styled(
        concat!("XRAT v", env!("CARGO_PKG_VERSION")),
        theme::accent_style().bold(),
    )];
    if let Some(latest) = &app.latest_version {
        brand.push(Span::styled(" → ", theme::muted_style()));
        brand.push(Span::styled(
            format!("v{}", latest.trim_start_matches('v')),
            theme::success_style().bold(),
        ));
    }

    let actions = [Span::styled("[?]Help", theme::chrome_style())];

    let total = area.width as usize;
    let brand_width: usize = brand.iter().map(Span::width).sum();
    let actions_width: usize = actions.iter().map(Span::width).sum();
    let gap = total
        .saturating_sub(brand_width + actions_width + theme::EDGE_MARGIN)
        .max(3);

    let mut spans = brand;
    spans.push(Span::raw(" ".repeat(gap)));
    spans.extend(actions);

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
