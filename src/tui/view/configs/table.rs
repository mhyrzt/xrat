use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Row, Table, TableState};

use crate::tui::app::TuiApp;
use crate::tui::theme;

const MAX_NAME_CHARS: usize = 24;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let header = Row::new([
        "St", "Ref", "Name", "Proto", "Address", "Port", "Net", "Delay",
    ])
    .style(theme::accent_style().add_modifier(Modifier::BOLD));

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

        Row::new(vec![
            Cell::from(state_marker(config)),
            Cell::from(config.display_ref().to_string()),
            Cell::from(truncate_name(config.display_name())),
            Cell::from(config.protocol.clone()),
            Cell::from(config.address_label().to_string()),
            Cell::from(config.port_label()),
            Cell::from(config.network_label()),
            Cell::from(delay_label(app, config)),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Max(MAX_NAME_CHARS as u16),
            Constraint::Length(9),
            Constraint::Percentage(30),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
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
        Span::styled(
            " [Configs]",
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Sources ", theme::muted_style()),
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
            format!("· src:{scope} "),
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

fn delay_label(app: &TuiApp, config: &crate::tui::data::TuiConfigRow) -> String {
    if app.is_testing_config(config.id) {
        const SPINNER: [&str; 4] = ["|", "/", "-", "\\"];
        SPINNER[app.spinner_tick % SPINNER.len()].to_string()
    } else {
        config.delay_label()
    }
}
