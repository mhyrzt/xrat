use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Modifier;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let header = Row::new(["ID", "Name", "Proto", "Address:Port", "Net", "Delay"])
        .style(theme::accent_style().add_modifier(Modifier::BOLD));

    let visible = app.visible_configs();
    let rows = visible.iter().enumerate().map(|(idx, config)| {
        let is_stale = config.real_delay_ms.is_none()
            && config.tcp_ms.is_none()
            && config.failure_reason.is_none()
            && config.is_enabled
            && !config.is_deleted;
        let mut style = if idx == app.config_list.focused {
            theme::accent_style().add_modifier(Modifier::BOLD)
        } else if config.is_active {
            theme::success_style().add_modifier(Modifier::BOLD)
        } else if config.is_selected {
            theme::accent_style()
        } else if !config.is_enabled || config.is_deleted {
            theme::muted_style()
        } else if config.failure_reason.is_some() {
            theme::failure_style()
        } else if is_stale {
            theme::warning_style()
        } else {
            theme::chrome_style()
        };

        if config.is_active {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        Row::new(vec![
            Cell::from(format!("{}{}", state_marker(config), config.id)),
            Cell::from(config.display_name().to_string()),
            Cell::from(config.protocol.clone()),
            Cell::from(config.endpoint()),
            Cell::from(config.network_label()),
            Cell::from(delay_label(app, config)),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Percentage(28),
            Constraint::Length(9),
            Constraint::Percentage(31),
            Constraint::Length(10),
            Constraint::Length(8),
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

fn state_marker(config: &crate::tui::data::TuiConfigRow) -> &'static str {
    if config.is_active {
        "*"
    } else if config.is_selected {
        ">"
    } else if config.is_deleted {
        "d"
    } else if !config.is_enabled {
        "x"
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
