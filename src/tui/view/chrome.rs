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
        ("3", "Tests", TuiView::Tests),
        ("4", "Diag", TuiView::Diagnostics),
    ];

    let mut spans = vec![
        Span::styled("XRAT ", theme::accent_style().bold()),
        Span::raw(" "),
        Span::styled("view:", theme::muted_style()),
        Span::raw(" "),
    ];

    for (idx, (key, label, view)) in modes.into_iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw("   "));
        }
        let style = if active_view == view {
            theme::accent_style().bold()
        } else {
            theme::chrome_style()
        };
        spans.push(Span::styled(format!("[{key}]{label}"), style));
    }

    spans.extend([
        Span::raw("        "),
        Span::styled("actions:", theme::muted_style()),
        Span::raw(" "),
        Span::styled("[?]Help", theme::chrome_style()),
        Span::raw("   "),
        Span::styled("[q]uit", theme::chrome_style()),
    ]);

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}
