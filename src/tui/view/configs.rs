use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::data::TuiConfigRow;
use crate::tui::theme;
use crate::tui::view::shared::detail_line;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(6)])
        .split(area);

    render_filter_strip(frame, sections[0], app);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(sections[1]);

    render_table(frame, columns[0], app);
    render_detail(frame, columns[1], app.focused_config());
}

fn render_filter_strip(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

fn render_table(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

fn render_detail(frame: &mut Frame<'_>, area: Rect, config: Option<&TuiConfigRow>) {
    let lines = match config {
        Some(config) => vec![
            Line::styled(
                format!("#{} {}", config.id, config.display_name()),
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            detail_line("Protocol", &config.protocol),
            detail_line("Endpoint", config.endpoint()),
            detail_line("Network", config.network_label()),
            detail_line("Real Delay", config.delay_label()),
            detail_line(
                "TCP",
                config
                    .tcp_ms
                    .map(|tcp| format!("{tcp}ms"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            detail_line(
                "Source",
                config
                    .source_id
                    .map(|source_id| format!("#{source_id}"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            detail_line("Status", config.status_label()),
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
