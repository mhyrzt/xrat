use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;

use crate::tui::app::TuiApp;
use crate::tui::data::{TuiConfigRow, TuiMetricColumns};
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, numbered_title, push_detail, render_scroll_panel};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let config = app.focused_config();
    let source_label = config
        .map(|config| source_label(app, config))
        .unwrap_or_default();
    let lines = detail_lines(area, config, &source_label, app.data.metric_columns);
    let viewport = render_scroll_panel(
        frame,
        area,
        lines,
        &app.panel_scroll.detail,
        PanelStyle {
            title: numbered_title(3, "Detail"),
            focused,
            right_pad: RIGHT_PAD,
            wrap_trim: true,
            header: None,
        },
    );
    app.panel_viewport.detail.set(viewport);
}

const LABEL_WIDTH: usize = crate::tui::theme::DETAIL_LABEL_WIDTH;
const RIGHT_PAD: u16 = crate::tui::theme::DETAIL_RIGHT_PAD;

fn source_label(app: &TuiApp, config: &TuiConfigRow) -> String {
    match config.source_id {
        Some(source_id) => app
            .data
            .sources
            .iter()
            .find(|source| source.id == source_id)
            .map(|source| format!("{} {}", source.display_ref(), source.display_name()))
            .unwrap_or_else(|| format!("#{source_id}")),
        None => "none".to_string(),
    }
}

fn detail_lines<'a>(
    area: Rect,
    config: Option<&'a TuiConfigRow>,
    source_label: &'a str,
    metric_columns: TuiMetricColumns,
) -> Vec<Line<'a>> {
    let content_width = area.width.saturating_sub(2 + RIGHT_PAD) as usize;
    match config {
        Some(config) => {
            let mut lines = vec![
                Line::styled(
                    format!("{} {}", config.display_ref(), config.display_name()),
                    theme::accent_style().add_modifier(Modifier::BOLD),
                ),
                Line::raw(""),
            ];
            push_detail(
                &mut lines,
                "ID",
                config.id.to_string(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(&mut lines, "Ref", &config.r#ref, LABEL_WIDTH, content_width);
            push_detail(
                &mut lines,
                "Protocol",
                &config.protocol,
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Endpoint",
                config.endpoint(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Network",
                config.network_label(),
                LABEL_WIDTH,
                content_width,
            );
            if metric_columns.icmp {
                push_detail(
                    &mut lines,
                    "ICMP",
                    config.icmp_label(),
                    LABEL_WIDTH,
                    content_width,
                );
            }
            if metric_columns.tcp {
                push_detail(
                    &mut lines,
                    "TCP",
                    config.tcp_label(),
                    LABEL_WIDTH,
                    content_width,
                );
            }
            if metric_columns.real_delay {
                push_detail(
                    &mut lines,
                    "Real Delay",
                    config.delay_label(),
                    LABEL_WIDTH,
                    content_width,
                );
            }
            if metric_columns.download {
                push_detail(
                    &mut lines,
                    "Download",
                    config.download_detail_label(),
                    LABEL_WIDTH,
                    content_width,
                );
            }
            if metric_columns.upload {
                push_detail(
                    &mut lines,
                    "Upload",
                    config.upload_detail_label(),
                    LABEL_WIDTH,
                    content_width,
                );
            }
            if metric_columns.country {
                push_detail(
                    &mut lines,
                    "Country",
                    config.country_label(),
                    LABEL_WIDTH,
                    content_width,
                );
                push_detail(
                    &mut lines,
                    "City",
                    config.location_label(),
                    LABEL_WIDTH,
                    content_width,
                );
                push_detail(
                    &mut lines,
                    "ASN",
                    config.asn_label(),
                    LABEL_WIDTH,
                    content_width,
                );
            }
            push_detail(
                &mut lines,
                "Subscription",
                source_label,
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Enabled",
                yes_no(config.is_enabled),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Active",
                yes_no(config.is_active),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Deleted",
                yes_no(config.is_deleted),
                LABEL_WIDTH,
                content_width,
            );
            lines.push(Line::raw(""));
            push_detail(
                &mut lines,
                "Failure",
                config.failure_reason.as_deref().unwrap_or("-"),
                LABEL_WIDTH,
                content_width,
            );
            lines
        }
        None => vec![
            Line::styled(
                "No configs",
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::raw("Import configs with `xrat import <input>`"),
            Line::raw("or add one with `xrat add <config-uri>`."),
        ],
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::*;

    fn row() -> TuiConfigRow {
        TuiConfigRow {
            id: 1,
            r#ref: "abcdef123456".to_string(),
            name: "main".to_string(),
            protocol: "vless".to_string(),
            address: "example.com".to_string(),
            port: 443,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            icmp_ms: Some(10),
            real_delay_ms: Some(100),
            tcp_ms: Some(20),
            download_mbps: Some(42.0),
            upload_mbps: Some(11.0),
            endpoint_country: Some("NL".to_string()),
            endpoint_location: Some("NL/Amsterdam".to_string()),
            endpoint_asn: Some("AS60781 LeaseWeb".to_string()),
            failure_reason: None,
            source_id: None,
            tested_at: Some("tested".to_string()),
            imported_at: "imported".to_string(),
            is_active: false,
            is_enabled: true,
            is_deleted: false,
        }
    }

    #[test]
    fn detail_lines_hide_disabled_metric_fields() {
        let metric_columns = TuiMetricColumns {
            icmp: false,
            tcp: true,
            real_delay: true,
            download: false,
            upload: false,
            country: false,
        };
        let row = row();
        let lines = detail_lines(Rect::new(0, 0, 80, 24), Some(&row), "none", metric_columns);
        let text = lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(!text.contains("ICMP"));
        assert!(text.contains("TCP"));
        assert!(text.contains("Real Delay"));
        assert!(!text.contains("Download"));
        assert!(!text.contains("Country"));
    }
}
