use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::{append_bottom_lines, detail_line};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let rt = &app.data.runtime;

    let proxy = match (&rt.socks, &rt.http) {
        (Some(s), _) => s.clone(),
        (_, Some(h)) => h.clone(),
        _ => "-".to_string(),
    };

    let active = rt.active_config.as_deref().unwrap_or("-");
    let selected = rt.selected_config.as_deref().unwrap_or("-");
    let task = app.task_state.label();

    let mut lines = vec![
        Line::styled(
            format!("Runtime  [{}]", rt.status),
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        detail_line("Active", active),
        detail_line("Selected", selected),
        detail_line("Task", &task),
        detail_line("Proxy", &proxy),
    ];

    if let Some(reason) = rt.failure_reason.as_deref() {
        lines.push(detail_line("Failure", reason));
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
