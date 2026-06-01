use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, Wrap};

use crate::tui::app::{TuiApp, TuiView};
use crate::tui::data::{TuiConfigRow, TuiSourceRow, TuiTestResultRow};
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    render_status_bar(frame, shell[0], app);
    render_body(frame, shell[1], app);
    render_key_bar(frame, shell[2]);

    if app.show_help {
        render_help(frame, centered_rect(68, 54, area));
    }

    if app.confirm.is_some() {
        render_confirm(frame, centered_rect(62, 34, area), app);
    }
}

fn render_status_bar(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

fn render_body(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(20)])
        .split(area);

    render_mode_rail(frame, columns[0], app.active_view);
    match app.active_view {
        TuiView::Configs => render_configs_view(frame, columns[1], app),
        TuiView::Sources => render_sources_view(frame, columns[1], app),
        TuiView::Runtime => render_runtime_view(frame, columns[1], app),
        TuiView::Tests => render_tests_view(frame, columns[1], app),
    }
}

fn render_mode_rail(frame: &mut Frame<'_>, area: Rect, active_view: TuiView) {
    let modes = [
        ("1", "Configs", TuiView::Configs),
        ("2", "Sources", TuiView::Sources),
        ("3", "Tests", TuiView::Tests),
        ("4", "Runtime", TuiView::Runtime),
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

fn render_configs_view(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(6)])
        .split(area);

    render_config_filter_strip(frame, sections[0], app);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(sections[1]);

    render_configs_table(frame, columns[0], app);
    render_config_detail(frame, columns[1], app.focused_config());
}

fn render_config_filter_strip(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let cursor = if app.config_list.editing_search {
        "_"
    } else {
        ""
    };
    let search = if app.config_list.search_query.is_empty() {
        "<none>".to_string()
    } else {
        format!("{}{}", app.config_list.search_query, cursor)
    };
    let text = Line::from(vec![
        Span::styled("Search: ", theme::muted_style()),
        Span::raw(search),
        Span::raw("   "),
        Span::styled("Sort: ", theme::muted_style()),
        Span::raw(app.config_list.sort.label()),
        Span::raw("   "),
        Span::styled("Visible: ", theme::muted_style()),
        Span::raw(app.visible_configs().len().to_string()),
        Span::raw("   / search  f deleted  s sort"),
    ]);

    frame.render_widget(
        Paragraph::new(text).block(Block::default().title(" Filter ").borders(Borders::ALL)),
        area,
    );
}

fn render_configs_table(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let header = Row::new(["ID", "Name", "Proto", "Address:Port", "Net", "Delay", "St"])
        .style(theme::accent_style().add_modifier(Modifier::BOLD));

    let visible = app.visible_configs();
    let rows = visible.iter().enumerate().map(|(idx, config)| {
        let mut style = if idx == app.config_list.focused {
            theme::accent_style().add_modifier(Modifier::BOLD)
        } else if !config.is_enabled || config.is_deleted {
            theme::muted_style()
        } else if config.failure_reason.is_some() {
            theme::failure_style()
        } else {
            theme::chrome_style()
        };

        if config.is_active {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        Row::new(vec![
            Cell::from(config.id.to_string()),
            Cell::from(config.display_name().to_string()),
            Cell::from(config.protocol.clone()),
            Cell::from(config.endpoint()),
            Cell::from(config.network_label()),
            Cell::from(config.delay_label()),
            Cell::from(config.status_label()),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(24),
            Constraint::Length(8),
            Constraint::Percentage(28),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(
                " Configs ({}/{}) ",
                visible.len(),
                app.data.total_configs
            ))
            .borders(Borders::ALL),
    )
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_config_detail(frame: &mut Frame<'_>, area: Rect, config: Option<&TuiConfigRow>) {
    let lines = match config {
        Some(config) => vec![
            Line::styled(
                format!("#{} {}", config.id, config.display_name()),
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            detail_line("Protocol", &config.protocol),
            detail_line("Endpoint", &config.endpoint()),
            detail_line("Network", &config.network_label()),
            detail_line("Real Delay", &config.delay_label()),
            detail_line(
                "TCP",
                &config
                    .tcp_ms
                    .map(|tcp| format!("{tcp}ms"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            detail_line(
                "Source",
                &config
                    .source_id
                    .map(|source_id| format!("#{source_id}"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            detail_line("Status", &config.status_label()),
            Line::raw(""),
            Line::styled("Actions", theme::muted_style()),
            Line::raw("Space select - e enable - x disable"),
            Line::raw("d delete - r restore - D purge - ? help"),
            Line::raw(""),
            detail_line("Failure", config.failure_reason.as_deref().unwrap_or("-")),
        ],
        None => vec![
            Line::styled(
                "No configs",
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::raw("Import configs with `xrat import <input>`"),
            Line::raw("or add one with `xrat add <config-uri>`."),
        ],
    };

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().title(" Detail ").borders(Borders::ALL))
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_sources_view(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_sources_table(frame, columns[0], app);
    render_source_detail(frame, columns[1], app.focused_source());
}

fn render_sources_table(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let header = Row::new(["ID", "Name", "Kind", "Configs", "Updated"])
        .style(theme::accent_style().add_modifier(Modifier::BOLD));

    let rows = app.data.sources.iter().enumerate().map(|(idx, source)| {
        let style = if idx == app.source_list.focused {
            theme::accent_style().add_modifier(Modifier::BOLD)
        } else {
            theme::chrome_style()
        };

        Row::new(vec![
            Cell::from(source.id.to_string()),
            Cell::from(source.display_name().to_string()),
            Cell::from(source.kind.clone()),
            Cell::from(source.config_count.to_string()),
            Cell::from(source.updated_at.clone()),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Sources ({}) ", app.data.sources.len()))
            .borders(Borders::ALL),
    )
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_source_detail(frame: &mut Frame<'_>, area: Rect, source: Option<&TuiSourceRow>) {
    let lines = match source {
        Some(source) => vec![
            Line::styled(
                format!("#{} {}", source.id, source.display_name()),
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            detail_line("Kind", &source.kind),
            detail_line("Value", source.value_label()),
            detail_line("Configs", source.config_count.to_string()),
            detail_line("Created", &source.created_at),
            detail_line("Updated", &source.updated_at),
            Line::raw(""),
            Line::styled("Actions", theme::muted_style()),
            Line::raw("r refresh focused - R refresh all"),
            Line::raw("i import - c copy - y QR (coming next)"),
        ],
        None => vec![
            Line::styled(
                "No sources",
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::raw("Import a subscription with `xrat import <input>`."),
        ],
    };

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Source Detail ")
                    .borders(Borders::ALL),
            )
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_runtime_view(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(8)])
        .split(area);

    render_runtime_status_cards(frame, sections[0], app);
    render_runtime_detail(frame, sections[1], app);
}

fn render_tests_view(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(8),
        ])
        .split(area);

    render_test_summary_cards(frame, sections[0], app);
    render_test_progress(frame, sections[1], app);
    render_test_results(frame, sections[2], app);
}

fn render_test_summary_cards(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

    render_runtime_card(
        frame,
        cards[0],
        " Scope ",
        &format!(
            "{} ({})",
            app.test_state.scope.label(),
            app.test_scope_count()
        ),
    );
    render_runtime_card(frame, cards[1], " Mode ", app.test_state.mode.label());
    render_runtime_card(
        frame,
        cards[2],
        " Latest Run ",
        &tests
            .latest_run_id
            .map(|id| format!("#{id}"))
            .unwrap_or_else(|| "-".to_string()),
    );
    render_runtime_card(
        frame,
        cards[3],
        " Queue ",
        &format!("{} concurrency", app.test_state.concurrency),
    );
}

fn render_test_progress(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

fn render_test_results(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    render_test_result_table(frame, columns[0], &app.data.tests.recent_results);
    render_test_detail(frame, columns[1], app);
}

fn render_test_result_table(frame: &mut Frame<'_>, area: Rect, results: &[TuiTestResultRow]) {
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

fn render_test_detail(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

fn render_runtime_status_cards(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let runtime = &app.data.runtime;
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    render_runtime_card(frame, cards[0], " Status ", &runtime.status);
    render_runtime_card(
        frame,
        cards[1],
        " PID ",
        &runtime
            .process_id
            .map(|pid| {
                if runtime.pid_running {
                    format!("{pid} running")
                } else {
                    format!("{pid} stopped")
                }
            })
            .unwrap_or_else(|| "-".to_string()),
    );
    render_runtime_card(
        frame,
        cards[2],
        " Active ",
        runtime.active_config.as_deref().unwrap_or("-"),
    );
    render_runtime_card(
        frame,
        cards[3],
        " Selected ",
        runtime.selected_config.as_deref().unwrap_or("-"),
    );
}

fn render_runtime_card(frame: &mut Frame<'_>, area: Rect, title: &'static str, value: &str) {
    frame.render_widget(
        Paragraph::new(value.to_string())
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_runtime_detail(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let runtime = &app.data.runtime;
    let lines = vec![
        Line::styled(
            "Runtime",
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        detail_line(
            "Session",
            runtime
                .session_id
                .map(|id| format!("#{id}"))
                .unwrap_or_else(|| "-".to_string()),
        ),
        detail_line(
            "Session Status",
            runtime.session_status.as_deref().unwrap_or("-"),
        ),
        detail_line(
            "Session Config",
            runtime.session_config.as_deref().unwrap_or("-"),
        ),
        detail_line("Socks", runtime.socks.as_deref().unwrap_or("-")),
        detail_line("HTTP", runtime.http.as_deref().unwrap_or("-")),
        detail_line("Shadowsocks", runtime.shadowsocks.as_deref().unwrap_or("-")),
        detail_line("Started", runtime.started_at.as_deref().unwrap_or("-")),
        detail_line("Stopped", runtime.stopped_at.as_deref().unwrap_or("-")),
        detail_line("Updated", runtime.updated_at.as_deref().unwrap_or("-")),
        detail_line("Failure", runtime.failure_reason.as_deref().unwrap_or("-")),
        detail_line(
            "Transition",
            runtime.transition_reason.as_deref().unwrap_or("-"),
        ),
        detail_line("Database", &runtime.database_label),
        Line::raw(""),
        Line::styled("Actions", theme::muted_style()),
        Line::raw("s start - x stop - r restart - l logs (coming next)"),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Runtime Detail ")
                    .borders(Borders::ALL),
            )
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn detail_line(label: &str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme::muted_style()),
        Span::raw(value.into()),
    ])
}

fn render_key_bar(frame: &mut Frame<'_>, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" Mode:", theme::muted_style()),
        Span::raw(" 1 configs  2 sources  3 tests  4 runtime   "),
        Span::styled("Move:", theme::muted_style()),
        Span::raw(" j/k arrows   "),
        Span::styled("Other:", theme::muted_style()),
        Span::raw(" / search  f deleted  ? help  q quit"),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_help(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(Clear, area);
    let text = vec![
        Line::styled("XRAT TUI Help", theme::accent_style().bold()),
        Line::raw(""),
        Line::raw("1-4       switch views"),
        Line::raw("j/k       move focus"),
        Line::raw("arrows    move focus"),
        Line::raw("/         edit config search"),
        Line::raw("f         show/hide deleted configs"),
        Line::raw("s         cycle config sort"),
        Line::raw("Space     select focused config"),
        Line::raw("e/x       enable/disable focused config"),
        Line::raw("d/D       soft delete / purge focused config"),
        Line::raw("r         restore focused deleted config"),
        Line::raw("Ctrl+U    clear search while editing"),
        Line::raw("Esc       close modal/back"),
        Line::raw("q/Ctrl+C  quit"),
    ];
    let block = Block::default().title(" Help ").borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Left)
            .style(theme::chrome_style()),
        area,
    );
}

fn render_confirm(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(confirm) = &app.confirm else {
        return;
    };

    frame.render_widget(Clear, area);
    let text = vec![
        Line::styled(&confirm.message, theme::chrome_style()),
        Line::raw(""),
        Line::styled("Enter/y confirm   Esc/n cancel", theme::muted_style()),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(confirm.title.as_str())
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}
