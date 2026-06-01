use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::tui::app::TuiApp;
use crate::tui::view::shared::render_card;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let tests = &app.data.tests;
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    render_card(
        frame,
        cards[0],
        " Scope ",
        &format!(
            "{} ({})",
            app.test_state.scope.label(),
            app.test_scope_count()
        ),
    );
    render_card(frame, cards[1], " Mode ", app.test_state.mode.label());
    render_card(
        frame,
        cards[2],
        " Latest Run ",
        &tests
            .latest_run_id
            .map(|id| format!("#{id}"))
            .unwrap_or_else(|| "-".to_string()),
    );
    render_card(
        frame,
        cards[3],
        " Queue ",
        &format!("{} concurrency", app.test_state.concurrency),
    );
}
