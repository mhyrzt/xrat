use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::TuiApp;
use crate::tui::keymap;
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

    let (center_text, center_style) = center_segment(app);
    let center_width = center_text.chars().count();

    let total = area.width as usize;
    let brand_width: usize = brand.iter().map(Span::width).sum();
    let actions_width: usize = actions.iter().map(Span::width).sum();
    let remaining = total
        .saturating_sub(brand_width + actions_width + center_width + theme::EDGE_MARGIN)
        .max(3);
    let left_gap = remaining / 2;
    let right_gap = remaining - left_gap;

    let mut spans = brand;
    spans.push(Span::raw(" ".repeat(left_gap)));
    if center_width > 0 {
        spans.push(Span::styled(center_text, center_style));
    }
    spans.push(Span::raw(" ".repeat(right_gap)));
    spans.extend(actions);

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Center segment of the key bar: an inline bulk confirm prompt takes priority,
/// then an armed chord hint, otherwise the latest status message.
fn center_segment(app: &TuiApp) -> (String, Style) {
    if let Some(op) = app.pending_bulk {
        let count = app.bulk_config_ids(op).len();
        return (
            format!("{} {count} {} configs? y/n", op.verb(), op.target()),
            theme::failure_style().bold(),
        );
    }
    if let Some(leader) = app.pending_chord {
        return (
            format!("[{leader}-] {}", keymap::chord_hint(leader)),
            theme::accent_style(),
        );
    }
    (String::new(), theme::muted_style())
}
