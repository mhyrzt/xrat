use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Table, TableState,
};

use crate::tui::app::TuiApp;
use crate::tui::theme;

const MAX_NAME_CHARS: usize = 36;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableColumnsLayout {
    pub show_proto: bool,
    pub show_address: bool,
    pub show_port: bool,
    pub show_net: bool,
    pub show_country: bool,
    pub show_icmp: bool,
    pub show_tcp: bool,
    pub show_real: bool,
    pub show_download: bool,
    pub show_upload: bool,
}

impl TableColumnsLayout {
    pub fn new(available_width: u16, metric_columns: &crate::tui::data::TuiMetricColumns) -> Self {
        let show_proto = available_width >= 52;
        let show_address = available_width >= 76;
        let show_port = available_width >= 96;
        let show_net = available_width >= 108;
        let show_country = metric_columns.country && available_width >= 66;

        let has_latency = metric_columns.real_delay || metric_columns.tcp || metric_columns.icmp;
        let primary_latency = if metric_columns.real_delay {
            Some("real")
        } else if metric_columns.tcp {
            Some("tcp")
        } else if metric_columns.icmp {
            Some("icmp")
        } else {
            None
        };

        let show_primary = has_latency && available_width >= 44;
        let show_secondary = available_width >= 86;
        let show_throughput = available_width >= 118;

        let show_real = metric_columns.real_delay
            && ((primary_latency == Some("real") && show_primary) || show_secondary);
        let show_tcp = metric_columns.tcp
            && ((primary_latency == Some("tcp") && show_primary) || show_secondary);
        let show_icmp = metric_columns.icmp
            && ((primary_latency == Some("icmp") && show_primary) || show_secondary);
        let show_download = metric_columns.download && show_throughput;
        let show_upload = metric_columns.upload && show_throughput;

        Self {
            show_proto,
            show_address,
            show_port,
            show_net,
            show_country,
            show_icmp,
            show_tcp,
            show_real,
            show_download,
            show_upload,
        }
    }
}

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    app.panel_viewport.table.set(area.height.saturating_sub(3));
    let metric_columns = app.data.metric_columns;
    let available_width = area.width.saturating_sub(4);
    let layout = TableColumnsLayout::new(available_width, &metric_columns);

    let mut headers = vec!["St", "Ref", "Name"];
    let mut constraints = vec![
        Constraint::Length(2),
        Constraint::Length(7),
        Constraint::Fill(1),
    ];
    if layout.show_proto {
        headers.push("Proto");
        constraints.push(Constraint::Length(7));
    }
    if layout.show_address {
        headers.push("Address");
        constraints.push(Constraint::Length(18));
    }
    if layout.show_port {
        headers.push("Port");
        constraints.push(Constraint::Length(5));
    }
    if layout.show_net {
        headers.push("Net");
        constraints.push(Constraint::Length(5));
    }
    if layout.show_icmp {
        headers.push("ICMP");
        constraints.push(Constraint::Length(7));
    }
    if layout.show_tcp {
        headers.push("TCP");
        constraints.push(Constraint::Length(7));
    }
    if layout.show_real {
        headers.push("Real");
        constraints.push(Constraint::Length(7));
    }
    if layout.show_download {
        headers.push("Down");
        constraints.push(Constraint::Length(8));
    }
    if layout.show_upload {
        headers.push("Up");
        constraints.push(Constraint::Length(8));
    }
    if layout.show_country {
        headers.push("Country");
        constraints.push(Constraint::Length(8));
    }
    let header = Row::new(headers).style(theme::secondary_style().add_modifier(Modifier::BOLD));

    let visible = app.visible_configs();
    let rows = visible.iter().enumerate().map(|(idx, config)| {
        let is_focused = idx == app.config_list.focused;
        let is_stale = config.real_delay_ms.is_none()
            && config.tcp_ms.is_none()
            && config.failure_reason.is_none()
            && config.is_enabled
            && !config.is_deleted;

        let row_style = if is_focused {
            theme::selection_style().add_modifier(Modifier::BOLD)
        } else if !config.is_enabled || config.is_deleted {
            theme::muted_style()
        } else {
            theme::chrome_style()
        };

        let state_style = if config.is_active {
            theme::success_style().add_modifier(Modifier::BOLD)
        } else if config.is_deleted || config.failure_reason.is_some() {
            theme::failure_style()
        } else if !config.is_enabled {
            theme::muted_style()
        } else if is_stale {
            theme::warning_style()
        } else {
            theme::chrome_style()
        };

        let proto_style = if !config.is_enabled || config.is_deleted {
            theme::muted_style()
        } else {
            theme::secondary_style()
        };

        let ref_style = if !config.is_enabled || config.is_deleted {
            theme::muted_style()
        } else {
            theme::secondary_style()
        };

        let name_style = if is_focused {
            theme::accent_style().add_modifier(Modifier::BOLD)
        } else if config.is_active {
            theme::success_style().add_modifier(Modifier::BOLD)
        } else if !config.is_enabled || config.is_deleted {
            theme::muted_style()
        } else if config.failure_reason.is_some() {
            theme::failure_style()
        } else {
            theme::chrome_style()
        };

        let mut cells = vec![
            Cell::from(state_marker(config)).style(state_style),
            Cell::from(config.display_ref().to_string()).style(ref_style),
            Cell::from(truncate_name(config.display_name())).style(name_style),
        ];
        if layout.show_proto {
            cells.push(Cell::from(config.protocol.clone()).style(proto_style));
        }
        if layout.show_address {
            cells.push(Cell::from(config.address_label().to_string()));
        }
        if layout.show_port {
            cells.push(Cell::from(config.port_label()).style(theme::muted_style()));
        }
        if layout.show_net {
            cells.push(Cell::from(config.network_label()).style(theme::muted_style()));
        }
        if layout.show_icmp {
            let label = test_metric_label(app, config, || config.icmp_label());
            let style = theme::latency_text_style(&label);
            cells.push(Cell::from(label).style(style));
        }
        if layout.show_tcp {
            let label = test_metric_label(app, config, || config.tcp_label());
            let style = theme::latency_text_style(&label);
            cells.push(Cell::from(label).style(style));
        }
        if layout.show_real {
            let label = test_metric_label(app, config, || config.delay_label());
            let style = theme::latency_text_style(&label);
            cells.push(Cell::from(label).style(style));
        }
        if layout.show_download {
            let label = test_metric_label(app, config, || config.download_label());
            let style = if label == "-" {
                theme::muted_style()
            } else {
                theme::series_style("download")
            };
            cells.push(Cell::from(label).style(style));
        }
        if layout.show_upload {
            let label = test_metric_label(app, config, || config.upload_label());
            let style = if label == "-" {
                theme::muted_style()
            } else {
                theme::series_style("upload")
            };
            cells.push(Cell::from(label).style(style));
        }
        if layout.show_country {
            cells.push(Cell::from(config.country_label().to_string()));
        }
        Row::new(cells).style(row_style)
    });

    let block = Block::default()
        .title(tab_title(app))
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(border_style(focused));

    crate::tui::theme::clear(frame, area);

    if visible.is_empty() {
        let msg = if app.data.total_configs == 0 {
            "No configs imported. Press 'i' to import a subscription or config link."
        } else {
            "No configs match current filter/search. Press Esc to reset search, or 'F' to cycle filter."
        };
        let empty_p = Paragraph::new(Line::styled(msg, theme::muted_style())).block(block);
        frame.render_widget(empty_p, area);
        return;
    }

    let table = Table::new(rows, constraints)
        .header(header)
        .highlight_symbol("▶ ")
        .highlight_spacing(HighlightSpacing::Always)
        .block(block)
        .column_spacing(1);

    let mut state = TableState::default()
        .with_offset(app.config_list.table_offset.get())
        .with_selected(Some(app.config_list.focused));
    frame.render_stateful_widget(table, area, &mut state);
    app.config_list.table_offset.set(state.offset());

    let viewport = area.height.saturating_sub(3) as usize;
    if visible.len() > viewport {
        let mut scrollbar_state = ScrollbarState::new(visible.len())
            .viewport_content_length(viewport)
            .position(app.config_list.focused);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(theme::muted_style())
                .begin_symbol(None)
                .end_symbol(None),
            area,
            &mut scrollbar_state,
        );
    }
}

fn border_style(focused: bool) -> ratatui::style::Style {
    theme::border_style(focused)
}

fn tab_title(app: &TuiApp) -> Line<'static> {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled("1:", theme::secondary_style().add_modifier(Modifier::BOLD)),
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
        .take(MAX_NAME_CHARS.saturating_sub(1))
        .chain("…".chars())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::data::TuiMetricColumns;

    #[test]
    fn table_columns_layout_scales_with_width() {
        let metrics = TuiMetricColumns {
            real_delay: true,
            tcp: true,
            country: true,
            download: true,
            upload: true,
            ..Default::default()
        };

        // Ultra compact: only St, Ref, Name, Real
        let layout_48 = TableColumnsLayout::new(48, &metrics);
        assert!(!layout_48.show_proto);
        assert!(!layout_48.show_address);
        assert!(!layout_48.show_port);
        assert!(!layout_48.show_net);
        assert!(!layout_48.show_country);
        assert!(layout_48.show_real);
        assert!(!layout_48.show_tcp);
        assert!(!layout_48.show_download);

        // Compact: Proto is added
        let layout_60 = TableColumnsLayout::new(60, &metrics);
        assert!(layout_60.show_proto);
        assert!(!layout_60.show_address);
        assert!(!layout_60.show_port);
        assert!(!layout_60.show_net);
        assert!(!layout_60.show_country);
        assert!(layout_60.show_real);

        // Medium-compact: Country is added
        let layout_70 = TableColumnsLayout::new(70, &metrics);
        assert!(layout_70.show_proto);
        assert!(!layout_70.show_address);
        assert!(layout_70.show_country);

        // Medium: Address is added
        let layout_80 = TableColumnsLayout::new(80, &metrics);
        assert!(layout_80.show_proto);
        assert!(layout_80.show_address);
        assert!(!layout_80.show_port);
        assert!(!layout_80.show_net);

        // Medium-wide: Port is added
        let layout_100 = TableColumnsLayout::new(100, &metrics);
        assert!(layout_100.show_address);
        assert!(layout_100.show_port);
        assert!(!layout_100.show_net);

        // Full width: Net and throughput metrics added
        let layout_125 = TableColumnsLayout::new(125, &metrics);
        assert!(layout_125.show_address);
        assert!(layout_125.show_port);
        assert!(layout_125.show_net);
        assert!(layout_125.show_download);
        assert!(layout_125.show_upload);
    }

    #[test]
    fn truncate_name_preserves_short_and_ellipsizes_long() {
        assert_eq!(truncate_name("short"), "short");
        let long = "a".repeat(50);
        let truncated = truncate_name(&long);
        assert_eq!(truncated.chars().count(), MAX_NAME_CHARS);
        assert!(truncated.ends_with('…'));
    }
}
