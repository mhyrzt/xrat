use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::detail_line;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let rt = &app.data.runtime;

    let proxy = match (&rt.socks, &rt.http) {
        (Some(s), _) => s.clone(),
        (_, Some(h)) => h.clone(),
        _ => "-".to_string(),
    };

    let active = rt.active_config.as_deref().unwrap_or("-");
    let selected = rt.selected_config.as_deref().unwrap_or("-");

    let mut lines = vec![
        Line::styled(
            format!("Runtime  [{}]", rt.status),
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        detail_line("Active", active),
        detail_line("Selected", selected),
        detail_line("Proxy", &proxy),
    ];

    if let Some(reason) = rt.failure_reason.as_deref() {
        lines.push(detail_line("Failure", reason));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "S start  0 stop  R restart  w switch",
        theme::muted_style(),
    ));

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().title(" Runtime ").borders(Borders::ALL))
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}
