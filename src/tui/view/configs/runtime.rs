use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, numbered_title, push_detail, render_scroll_panel};

const LABEL_WIDTH: usize = crate::tui::theme::DETAIL_LABEL_WIDTH;
const RIGHT_PAD: u16 = crate::tui::theme::DETAIL_RIGHT_PAD;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let rt = &app.data.runtime;

    let active = rt.active_config.as_deref().unwrap_or("-");
    let in_flight = app.runtime_activity_in_flight();
    let task = if in_flight {
        if app.runtime_op_in_flight() {
            format!("{} {}", app.spinner_frame(), app.task_state.label())
        } else {
            format!("{} runtime transitioning", app.spinner_frame())
        }
    } else if app.subscriptions_refresh_in_flight() {
        format!("{} refreshing subscriptions", app.spinner_frame())
    } else {
        app.task_state.label()
    };
    let data = &app.data;
    let content_width = area.width.saturating_sub(2 + RIGHT_PAD) as usize;

    let status_line = if in_flight {
        format!("{} [{}]", app.spinner_frame(), rt.status)
    } else {
        format!("[{}]", rt.status)
    };
    let mut lines = vec![
        Line::styled(
            status_line,
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
    ];
    push_detail(&mut lines, "Active", active, LABEL_WIDTH, content_width);
    push_detail(&mut lines, "Task", &task, LABEL_WIDTH, content_width);

    let inbounds = [
        ("SOCKS", &rt.socks),
        ("HTTP", &rt.http),
        ("Shadowsocks", &rt.shadowsocks),
    ];
    let any_inbound = inbounds.iter().any(|(_, value)| value.is_some());
    if any_inbound {
        for (label, value) in inbounds {
            if let Some(endpoint) = value {
                push_detail(&mut lines, label, endpoint, LABEL_WIDTH, content_width);
            }
        }
    } else {
        push_detail(&mut lines, "Proxy", "-", LABEL_WIDTH, content_width);
    }

    let engines = if app.engines.is_empty() {
        "-".to_string()
    } else {
        app.engines
            .iter()
            .map(|engine| match (engine.available, &engine.version) {
                (true, Some(version)) => format!("{} {} ✓", engine.name, version),
                (true, None) => format!("{} ✓", engine.name),
                (false, _) => format!("{} ✗", engine.name),
            })
            .collect::<Vec<_>>()
            .join(" · ")
    };
    push_detail(&mut lines, "Engines", &engines, LABEL_WIDTH, content_width);

    push_detail(
        &mut lines,
        "Daemon",
        if data.daemon.running {
            "running"
        } else {
            "stopped"
        },
        LABEL_WIDTH,
        content_width,
    );
    if data.daemon.running {
        let rotation = if data.daemon.rotation_enabled {
            format!("on · every {}s", data.daemon.interval_secs)
        } else {
            "off".to_string()
        };
        push_detail(
            &mut lines,
            "Rotation",
            &rotation,
            LABEL_WIDTH,
            content_width,
        );
    }

    let config_stats = [
        (data.total_configs, "total"),
        (data.enabled_configs, "enabled"),
        (data.deleted_configs, "deleted"),
        (data.failed_configs, "failed"),
    ]
    .into_iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, label)| format!("{count} {label}"))
    .collect::<Vec<_>>()
    .join(" · ");
    let config_stats = if config_stats.is_empty() {
        "-".to_string()
    } else {
        config_stats
    };
    push_detail(
        &mut lines,
        "Configs",
        &config_stats,
        LABEL_WIDTH,
        content_width,
    );
    let subscriptions_count = if app.subscriptions_refresh_in_flight() {
        format!("{} {}", app.spinner_frame(), data.sources.len())
    } else {
        data.sources.len().to_string()
    };
    push_detail(
        &mut lines,
        "Subscriptions",
        subscriptions_count,
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Database",
        data.db_label.as_str(),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Config file",
        data.config_path.as_str(),
        LABEL_WIDTH,
        content_width,
    );
    if data.server_enabled {
        push_detail(
            &mut lines,
            "API sub URL",
            data.api_b64_url.as_str(),
            LABEL_WIDTH,
            content_width,
        );
    }

    if let Some(reason) = rt.failure_reason.as_deref() {
        push_detail(&mut lines, "Failure", reason, LABEL_WIDTH, content_width);
    }

    let viewport = render_scroll_panel(
        frame,
        area,
        lines,
        &app.panel_scroll.runtime,
        PanelStyle {
            title: numbered_title(4, "Runtime"),
            focused,
            right_pad: RIGHT_PAD,
            wrap_trim: true,
            header: None,
        },
    );
    app.panel_viewport.runtime.set(viewport);
}
