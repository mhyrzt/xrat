use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::{append_bottom_lines, push_detail};

const LABEL_WIDTH: usize = 13;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let rt = &app.data.runtime;

    let proxy = match (&rt.socks, &rt.http) {
        (Some(s), _) => s.clone(),
        (_, Some(h)) => h.clone(),
        _ => "-".to_string(),
    };

    let active = rt.active_config.as_deref().unwrap_or("-");
    let task = app.task_state.label();
    let data = &app.data;
    let content_width = area.width.saturating_sub(2) as usize;

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
    push_detail(
        &mut lines,
        "Configs",
        format!(
            "{} total  {} enabled  {} deleted  {} failed",
            data.total_configs, data.enabled_configs, data.deleted_configs, data.failed_configs,
        ),
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
    push_detail(
        &mut lines,
        "API sub URL",
        data.api_b64_url.as_str(),
        LABEL_WIDTH,
        content_width,
    );
    push_detail(
        &mut lines,
        "Log entries",
        app.event_log.len().to_string(),
        LABEL_WIDTH,
        content_width,
    );

    if let Some(reason) = rt.failure_reason.as_deref() {
        push_detail(&mut lines, "Failure", reason, LABEL_WIDTH, content_width);
    }

    append_bottom_lines(
        &mut lines,
        vec![
            Line::styled("Actions", theme::muted_style()),
            Line::raw("[K]ill  [R]estart"),
        ],
        area,
        2,
    );

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().title(" Runtime ").borders(Borders::ALL))
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}
