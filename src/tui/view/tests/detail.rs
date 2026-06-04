use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::push_detail;

const LABEL_WIDTH: usize = 16;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let tests = &app.data.tests;
    let content_width = area.width.saturating_sub(2) as usize;

    let mut lines = vec![
        Line::styled("Tests", theme::accent_style().add_modifier(Modifier::BOLD)),
        Line::raw(""),
    ];
    push_detail(
        &mut lines,
        "Scope",
        app.test_state.scope.label(),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Scope Count",
        app.test_scope_count().to_string(),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Mode",
        app.test_state.mode.label(),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Concurrency",
        app.test_state.concurrency.to_string(),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Latest Run",
        tests
            .latest_run_id
            .map(|id| format!("#{id}"))
            .unwrap_or_else(|| "-".to_string()),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Latest Kind",
        tests.latest_run_kind.as_deref().unwrap_or("-"),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Latest Created",
        tests.latest_run_created_at.as_deref().unwrap_or("-"),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Untested",
        tests.untested_configs.to_string(),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Failed/Stale",
        tests.stale_configs.to_string(),
        LABEL_WIDTH,
        content_width,
    );
    lines.push(Line::raw(""));
    lines.push(Line::styled("Actions", theme::muted_style()));
    lines.push(Line::raw("[s]tart  [c]ancel running batch"));

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Test Detail ")
                    .borders(Borders::ALL),
            )
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}
