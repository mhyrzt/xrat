use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::TuiView;
use crate::tui::theme;

pub fn render_mode_rail(frame: &mut Frame<'_>, area: Rect, active_view: TuiView) {
    let modes = [
        ("1", "Configs", TuiView::Configs),
        ("2", "Sources", TuiView::Sources),
        ("3", "Tests", TuiView::Tests),
        ("4", "Diagnostics", TuiView::Diagnostics),
    ];
    let lines: Vec<Line<'_>> = modes
        .into_iter()
        .map(|(key, label, view)| {
            let marker = if active_view == view { ">" } else { " " };
            let style = if active_view == view {
                theme::accent_style().bold()
            } else {
                theme::chrome_style()
            };
            Line::from(vec![
                Span::raw(marker),
                Span::raw(" "),
                Span::raw("["),
                Span::styled(key, style),
                Span::raw("] "),
                Span::styled(label.to_uppercase(), style),
            ])
        })
        .collect();

    let block = Block::default().title(" Modes ").borders(Borders::ALL);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn render_key_bar(frame: &mut Frame<'_>, area: Rect) {
    let spans = vec![
        Span::styled("XRAT ", theme::accent_style().bold()),
        Span::raw("  "),
        Span::styled("[?]help", theme::chrome_style()),
        Span::raw("  "),
        Span::styled("[q]quit", theme::chrome_style()),
    ];
    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}
