use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, push_detail, render_scroll_panel};

const LABEL_WIDTH: usize = crate::tui::theme::DETAIL_LABEL_WIDTH;
const RIGHT_PAD: u16 = crate::tui::theme::DETAIL_RIGHT_PAD;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let rt = &app.data.runtime;

    let proxy = match (&rt.socks, &rt.http) {
        (Some(s), _) => s.clone(),
        (_, Some(h)) => h.clone(),
        _ => "-".to_string(),
    };

    let active = rt.active_config.as_deref().unwrap_or("-");
    let task = app.task_state.label();
    let data = &app.data;
    let content_width = area.width.saturating_sub(2 + RIGHT_PAD) as usize;

    let mut lines = vec![
        Line::styled(
            format!("[{}]", rt.status),
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
    ];
    push_detail(&mut lines, "Active", active, LABEL_WIDTH, content_width);
    push_detail(&mut lines, "Task", &task, LABEL_WIDTH, content_width);
    push_detail(&mut lines, "Proxy", &proxy, LABEL_WIDTH, content_width);

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
    push_detail(
        &mut lines,
        "Sources",
        data.sources.len().to_string(),
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

    render_scroll_panel(
        frame,
        area,
        lines,
        &app.panel_scroll.runtime,
        PanelStyle {
            title: " Runtime ",
            focused,
            right_pad: RIGHT_PAD,
            wrap_trim: true,
        },
    );
}
