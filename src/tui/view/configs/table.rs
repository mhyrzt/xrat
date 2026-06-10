use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Row, Table, TableState};

use crate::tui::app::TuiApp;
use crate::tui::theme;

const MAX_NAME_CHARS: usize = 24;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    app.panel_viewport.table.set(area.height.saturating_sub(3));
    let metric_columns = app.data.metric_columns;
    let mut headers = vec!["St", "Ref", "Name", "Proto", "Address", "Port", "Net"];
    if metric_columns.icmp {
        headers.push("ICMP");
    }
    if metric_columns.tcp {
        headers.push("TCP");
    }
    if metric_columns.real_delay {
        headers.push("Real");
    }
    if metric_columns.download {
        headers.push("Down");
    }
    if metric_columns.upload {
        headers.push("Up");
    }
    if metric_columns.country {
        headers.push("Country");
    }
    let header = Row::new(headers).style(theme::accent_style().add_modifier(Modifier::BOLD));

    let visible = app.visible_configs();
    let rows = visible.iter().enumerate().map(|(idx, config)| {
        let is_stale = config.real_delay_ms.is_none()
            && config.tcp_ms.is_none()
            && config.failure_reason.is_none()
            && config.is_enabled
            && !config.is_deleted;
        let style = if idx == app.config_list.focused {
            theme::accent_style().add_modifier(Modifier::BOLD)
        } else if config.is_active {
            theme::success_style().add_modifier(Modifier::BOLD)
        } else if !config.is_enabled || config.is_deleted {
            theme::muted_style()
        } else if config.failure_reason.is_some() {
            theme::failure_style()
        } else if is_stale {
            theme::warning_style()
        } else {
            theme::chrome_style()
        };

        let mut cells = vec![
            Cell::from(state_marker(config)),
            Cell::from(config.display_ref().to_string()),
            Cell::from(truncate_name(config.display_name())),
            Cell::from(config.protocol.clone()),
            Cell::from(config.address_label().to_string()),
            Cell::from(config.port_label()),
            Cell::from(config.network_label()),
        ];
        if metric_columns.icmp {
            cells.push(Cell::from(test_metric_label(app, config, || {
                config.icmp_label()
            })));
        }
        if metric_columns.tcp {
            cells.push(Cell::from(test_metric_label(app, config, || {
                config.tcp_label()
            })));
        }
        if metric_columns.real_delay {
            cells.push(Cell::from(test_metric_label(app, config, || {
                config.delay_label()
            })));
        }
        if metric_columns.download {
            cells.push(Cell::from(test_metric_label(app, config, || {
                config.download_label()
            })));
        }
        if metric_columns.upload {
            cells.push(Cell::from(test_metric_label(app, config, || {
                config.upload_label()
            })));
        }
        if metric_columns.country {
            cells.push(Cell::from(config.country_label().to_string()));
        }
        Row::new(cells).style(style)
    });

    let mut constraints = vec![
        Constraint::Length(2),
        Constraint::Length(8),
        Constraint::Max(MAX_NAME_CHARS as u16),
        Constraint::Length(9),
        Constraint::Percentage(30),
        Constraint::Length(6),
        Constraint::Length(10),
    ];
    if metric_columns.icmp {
        constraints.push(Constraint::Length(8));
    }
    if metric_columns.tcp {
        constraints.push(Constraint::Length(8));
    }
    if metric_columns.real_delay {
        constraints.push(Constraint::Length(8));
    }
    if metric_columns.download {
        constraints.push(Constraint::Length(10));
    }
    if metric_columns.upload {
        constraints.push(Constraint::Length(10));
    }
    if metric_columns.country {
        constraints.push(Constraint::Length(10));
    }

    let table = Table::new(rows, constraints)
        .header(header)
        .block(
            Block::default()
                .title(tab_title(app))
                .borders(Borders::ALL)
                .border_style(border_style(focused)),
        )
        .column_spacing(1);

    let mut state = TableState::default().with_selected(Some(app.config_list.focused));
    frame.render_widget(Clear, area);
    frame.render_stateful_widget(table, area, &mut state);
}

fn border_style(focused: bool) -> ratatui::style::Style {
    if focused {
        theme::accent_style()
    } else {
        theme::chrome_style()
    }
}

fn tab_title(app: &TuiApp) -> Line<'static> {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled("1:", theme::accent_style().add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(
            "[Configs]",
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Subscriptions ", theme::muted_style()),
        Span::styled(
            format!(
                "({}/{}) ",
                app.visible_configs().len(),
                app.data.total_configs
            ),
            theme::chrome_style(),
        ),
    ];

    let scope = match app.config_list.source_filter {
        crate::tui::app::SourceFilter::All => None,
        crate::tui::app::SourceFilter::Orphans => Some("orphans".to_string()),
        crate::tui::app::SourceFilter::Source(source_id) => Some(
            app.data
                .sources
                .iter()
                .find(|source| source.id == source_id)
                .map(|source| format!("{} {}", source.display_ref(), source.display_name()))
                .unwrap_or_else(|| format!("#{source_id}")),
        ),
    };
    if let Some(scope) = scope {
        spans.push(Span::styled(
            format!("· sub:{scope} "),
            theme::accent_style(),
        ));
    }

    Line::from(spans)
}

fn truncate_name(name: &str) -> String {
    let char_count = name.chars().count();
    if char_count <= MAX_NAME_CHARS {
        return name.to_string();
    }

    name.chars()
        .take(MAX_NAME_CHARS.saturating_sub(3))
        .chain("...".chars())
        .collect()
}

fn state_marker(config: &crate::tui::data::TuiConfigRow) -> &'static str {
    if config.is_active {
        "●"
    } else if config.is_deleted {
        "✕"
    } else if !config.is_enabled {
        "○"
    } else if config.failure_reason.is_some() {
        "!"
    } else {
        " "
    }
}

fn test_metric_label(
    app: &TuiApp,
    config: &crate::tui::data::TuiConfigRow,
    label: impl FnOnce() -> String,
) -> String {
    if app.is_testing_config(config.id) {
        const SPINNER: [&str; 4] = ["|", "/", "-", "\\"];
        SPINNER[app.spinner_tick % SPINNER.len()].to_string()
    } else {
        label()
    }
}
