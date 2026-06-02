use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Gauge};

use crate::tui::app::TuiApp;
use crate::tui::task::TuiTaskKind;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let (ratio, label) = live_progress(app);
    let gauge = Gauge::default()
        .block(Block::default().title(" Progress ").borders(Borders::ALL))
        .gauge_style(theme::success_style())
        .label(label)
        .ratio(ratio);
    frame.render_widget(gauge, area);
}

fn live_progress(app: &TuiApp) -> (f64, String) {
    let task = &app.task_state;
    if task.running == Some(TuiTaskKind::TestBatch) && task.progress_total > 0 {
        let done = task.progress_done;
        let total = task.progress_total;
        let ratio = done as f64 / total as f64;
        let label = format!("{done}/{total} tested");
        return (ratio, label);
    }
    let tests = &app.data.tests;
    let ratio = if tests.total_results == 0 {
        0.0
    } else {
        tests.success_results as f64 / tests.total_results as f64
    };
    (ratio, tests.progress_label())
}
