use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::{TuiApp, TuiView};
use crate::tui::theme;

pub fn render_status_bar(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let line = Line::from(vec![
        Span::styled(" XRAT ", theme::accent_style().bold()),
        Span::styled(app.active_view.badge(), theme::chrome_style()),
        Span::raw(format!(
            " {} total - {} on - {} sel - {} del - {} fail - {}",
            app.data.total_configs,
            app.data.enabled_configs,
            app.data.selected_configs,
            app.data.deleted_configs,
            app.data.failed_configs,
            app.config_filter_summary()
        )),
        Span::raw("   "),
        Span::styled(
            format!("* {}", app.task_state.label()),
            theme::success_style().bold(),
        ),
        Span::raw("   "),
        Span::styled(&app.status_message, theme::muted_style()),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

pub fn render_mode_rail(frame: &mut Frame<'_>, area: Rect, active_view: TuiView) {
    let modes = [
        ("1", "Configs", TuiView::Configs),
        ("2", "Sources", TuiView::Sources),
        ("3", "Tests", TuiView::Tests),
        ("4", "Runtime", TuiView::Runtime),
        ("5", "Diagnostics", TuiView::Diagnostics),
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
                Span::styled(key, style),
                Span::raw(" "),
                Span::styled(label, style),
            ])
        })
        .collect();

    let block = Block::default().title(" Modes ").borders(Borders::ALL);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn render_key_bar(frame: &mut Frame<'_>, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" Mode:", theme::muted_style()),
        Span::raw(" 1 configs  2 sources  3 tests  4 runtime  5 diag   "),
        Span::styled("Move:", theme::muted_style()),
        Span::raw(" j/k arrows   "),
        Span::styled("Other:", theme::muted_style()),
        Span::raw(" / search  f deleted  ? help  q quit"),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
