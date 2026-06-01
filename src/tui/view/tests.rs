use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::data::TuiTestResultRow;
use crate::tui::theme;
use crate::tui::view::shared::{detail_line, render_card};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(8),
        ])
        .split(area);

    render_summary_cards(frame, sections[0], app);
    render_progress(frame, sections[1], app);
    render_results(frame, sections[2], app);
}

fn render_summary_cards(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

fn render_progress(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

fn render_results(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    render_result_table(frame, columns[0], &app.data.tests.recent_results);
    render_detail(frame, columns[1], app);
}

fn render_result_table(frame: &mut Frame<'_>, area: Rect, results: &[TuiTestResultRow]) {
    let header = Row::new(["ID", "Config", "Status", "Real", "TCP", "Tested"])
        .style(theme::accent_style().add_modifier(Modifier::BOLD));
    let rows = results.iter().map(|result| {
        let style = if result.failure_reason.is_some() {
            theme::failure_style()
        } else {
            theme::chrome_style()
        };
        Row::new(vec![
            Cell::from(result.id.to_string()),
            Cell::from(format!("#{}", result.config_id)),
            Cell::from(result.status.clone()),
            Cell::from(
                result
                    .real_delay_ms
                    .map(|delay| format!("{delay}ms"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::from(
                result
                    .tcp_ms
                    .map(|delay| format!("{delay}ms"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::from(result.tested_at.clone()),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Recent Results ")
            .borders(Borders::ALL),
    )
    .column_spacing(1);
    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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
        Line::raw("s start - c cancel (coming next)"),
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
