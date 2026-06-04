use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::TuiView;
use crate::tui::theme;

pub fn render_key_bar(frame: &mut Frame<'_>, area: Rect, active_view: TuiView) {
    let modes = [
        ("1", "Configs", TuiView::Configs),
        ("2", "Sources", TuiView::Sources),
    ];

    let mut views = vec![Span::styled("view:", theme::muted_style()), Span::raw(" ")];

    for (idx, (key, label, view)) in modes.into_iter().enumerate() {
        if idx > 0 {
            views.push(Span::raw("   "));
        }
        let style = if active_view == view {
            theme::accent_style().bold()
        } else {
            theme::chrome_style()
        };
        views.push(Span::styled(format!("[{key}]{label}"), style));
    }

    let center = Span::styled("XRAT", theme::accent_style().bold());

    let actions = [
        Span::styled("actions:", theme::muted_style()),
        Span::raw(" "),
        Span::styled("[?]Help", theme::chrome_style()),
        Span::raw("   "),
        Span::styled("[q]uit", theme::chrome_style()),
    ];

    let total = area.width as usize;
    let views_width: usize = views.iter().map(Span::width).sum();
    let center_width = center.width();
    let actions_width: usize = actions.iter().map(Span::width).sum();

    let center_start = total.saturating_sub(center_width) / 2;
    let left_gap = center_start.saturating_sub(views_width).max(3);
    let right_gap = center_start.saturating_sub(actions_width).max(3);

    let mut spans = views;
    spans.push(Span::raw(" ".repeat(left_gap)));
    spans.push(center);
    spans.push(Span::raw(" ".repeat(right_gap)));
    spans.extend(actions);

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}
