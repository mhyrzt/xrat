use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph};

use crate::tui::app::TuiApp;
use crate::tui::task::TuiTaskKind;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let block = Block::default()
        .title(" Testing Progress ")
        .borders(Borders::ALL);
    let inner = block.inner(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let summary = summary_label(app);
    let (ratio, gauge_label) = live_progress(app);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(summary.chars().count() as u16 + 1),
            Constraint::Min(8),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::styled(summary, theme::muted_style())),
        columns[0],
    );
    frame.render_widget(
        Gauge::default()
            .gauge_style(theme::success_style())
            .label(gauge_label)
            .ratio(ratio),
        columns[1],
    );
}

fn summary_label(app: &TuiApp) -> String {
    let test = &app.test_state;
    let mut summary = format!(
        "{} ({})  ·  {}  ·  c{}",
        test.scope.label(),
        app.test_scope_count(),
        test.mode.label(),
        test.concurrency,
    );

    if app.task_state.running != Some(TuiTaskKind::TestBatch) {
        let tests = &app.data.tests;
        if let Some(run_id) = tests.latest_run_id {
            let created = tests.latest_run_created_at.as_deref().unwrap_or("-");
            summary.push_str(&format!("  ·  #{run_id} {created}"));
        }
    }

    summary
}

fn live_progress(app: &TuiApp) -> (f64, String) {
    let task = &app.task_state;
    if task.running == Some(TuiTaskKind::TestBatch) && task.progress_total > 0 {
        let done = task.progress_done;
        let total = task.progress_total;
        let ratio = done as f64 / total as f64;
        return (ratio, format!("{done}/{total}"));
    }

    let tests = &app.data.tests;
    let ratio = if tests.total_results == 0 {
        0.0
    } else {
        tests.success_results as f64 / tests.total_results as f64
    };
    (ratio, tests.progress_label())
}
