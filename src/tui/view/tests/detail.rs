use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::detail_line;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let tests = &app.data.tests;
    let lines = vec![
        Line::styled("Tests", theme::accent_style().add_modifier(Modifier::BOLD)),
        Line::raw(""),
        detail_line("Scope", app.test_state.scope.label()),
        detail_line("Scope Count", app.test_scope_count().to_string()),
        detail_line("Mode", app.test_state.mode.label()),
        detail_line("Concurrency", app.test_state.concurrency.to_string()),
        detail_line(
            "Latest Kind",
            tests.latest_run_kind.as_deref().unwrap_or("-"),
        ),
        detail_line(
            "Latest Created",
            tests.latest_run_created_at.as_deref().unwrap_or("-"),
        ),
        detail_line("Untested", tests.untested_configs.to_string()),
        detail_line("Failed/Stale", tests.stale_configs.to_string()),
        Line::raw(""),
        Line::styled("Actions", theme::muted_style()),
        Line::raw("[s]tart  [c]ancel running batch"),
    ];

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
