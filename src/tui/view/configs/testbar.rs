use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph};

use crate::tui::app::TuiApp;
use crate::tui::task::TuiTaskKind;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let block = Block::default().title(" Testing ").borders(Borders::ALL);
    let inner = block.inner(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let summary = summary_label(app);
    let (ratio, gauge_label, result_label) = progress_labels(app);
    let result_width = result_label
        .as_ref()
        .map(|line| line.width().saturating_add(1).min(u16::MAX as usize) as u16);

    let mut constraints = vec![
        Constraint::Length(summary.chars().count() as u16 + 1),
        Constraint::Min(8),
    ];
    if let Some(width) = result_width {
        constraints.push(Constraint::Length(width));
    }

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
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
    if let Some(label) = result_label {
        frame.render_widget(Paragraph::new(label), columns[2]);
    }
}

fn summary_label(app: &TuiApp) -> String {
    let test = &app.test_state;
    format!(
        "{} ({})  ·  {}  ·  c{}",
        test.scope.label(),
        app.test_scope_count(),
        test.mode.label(),
        test.concurrency,
    )
}

fn progress_labels(app: &TuiApp) -> (f64, Span<'static>, Option<Line<'static>>) {
    let task = &app.task_state;
    if task.running == Some(TuiTaskKind::TestBatch) && task.progress_total > 0 {
        let done = task.progress_done;
        let total = task.progress_total;
        let ratio = done as f64 / total as f64;
        return (
            ratio,
            Span::raw(""),
            Some(Line::from(Span::styled(
                format!("{done}/{total}"),
                theme::success_style(),
            ))),
        );
    }

    // Idle (no live batch): leave the gauge empty; the result label still
    // reports the last persisted run's health.
    let tests = &app.data.tests;
    if tests.latest_run_id.is_none() {
        return (
            0.0,
            Span::styled(tests.progress_label(), theme::muted_style()),
            None,
        );
    }

    (0.0, Span::raw(""), Some(result_label(tests)))
}

fn result_label(tests: &crate::tui::data::TuiTestStatus) -> Line<'static> {
    let mut spans = Vec::new();
    push_result_part(
        &mut spans,
        tests.total_results,
        "done",
        theme::chrome_style(),
    );
    push_result_part(
        &mut spans,
        tests.success_results,
        "ok",
        theme::success_style(),
    );
    push_result_part(
        &mut spans,
        tests.failed_results,
        "failed",
        theme::failure_style(),
    );

    if spans.is_empty() {
        Line::styled("no results", theme::muted_style())
    } else {
        Line::from(spans)
    }
}

fn push_result_part(
    spans: &mut Vec<Span<'static>>,
    value: usize,
    label: &'static str,
    style: ratatui::style::Style,
) {
    if value == 0 {
        return;
    }
    if !spans.is_empty() {
        spans.push(Span::styled(" · ", theme::muted_style()));
    }
    spans.push(Span::styled(format!("{value} {label}"), style));
}
