use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Gauge};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let tests = &app.data.tests;
    let ratio = if tests.total_results == 0 {
        0.0
    } else {
        tests.success_results as f64 / tests.total_results as f64
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Latest Progress ")
                .borders(Borders::ALL),
        )
        .gauge_style(theme::success_style())
        .label(tests.progress_label())
        .ratio(ratio);
    frame.render_widget(gauge, area);
}
