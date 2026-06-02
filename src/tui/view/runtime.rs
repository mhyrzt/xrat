use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::{detail_line, render_card};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(8)])
        .split(area);

    render_status_cards(frame, sections[0], app);
    render_detail(frame, sections[1], app);
}

fn render_status_cards(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let runtime = &app.data.runtime;
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    render_card(frame, cards[0], " Status ", &runtime.status);
    render_card(
        frame,
        cards[1],
        " PID ",
        &runtime
            .process_id
            .map(|pid| {
                if runtime.pid_running {
                    format!("{pid} running")
                } else {
                    format!("{pid} stopped")
                }
            })
            .unwrap_or_else(|| "-".to_string()),
    );
    render_card(
        frame,
        cards[2],
        " Active ",
        runtime.active_config.as_deref().unwrap_or("-"),
    );
    render_card(
        frame,
        cards[3],
        " Selected ",
        runtime.selected_config.as_deref().unwrap_or("-"),
    );
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let runtime = &app.data.runtime;
    let lines = vec![
        Line::styled(
            "Runtime",
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        detail_line(
            "Session",
            runtime
                .session_id
                .map(|id| format!("#{id}"))
                .unwrap_or_else(|| "-".to_string()),
        ),
        detail_line(
            "Session Status",
            runtime.session_status.as_deref().unwrap_or("-"),
        ),
        detail_line(
            "Session Config",
            runtime.session_config.as_deref().unwrap_or("-"),
        ),
        detail_line("Socks", runtime.socks.as_deref().unwrap_or("-")),
        detail_line("HTTP", runtime.http.as_deref().unwrap_or("-")),
        detail_line("Shadowsocks", runtime.shadowsocks.as_deref().unwrap_or("-")),
        detail_line("Started", runtime.started_at.as_deref().unwrap_or("-")),
        detail_line("Stopped", runtime.stopped_at.as_deref().unwrap_or("-")),
        detail_line("Updated", runtime.updated_at.as_deref().unwrap_or("-")),
        detail_line("Failure", runtime.failure_reason.as_deref().unwrap_or("-")),
        detail_line(
            "Transition",
            runtime.transition_reason.as_deref().unwrap_or("-"),
        ),
        detail_line("Database", &runtime.database_label),
        Line::raw(""),
        Line::styled("Actions", theme::muted_style()),
        Line::raw("s start/connect  x stop/disconnect  r restart  w switch config"),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Runtime Detail ")
                    .borders(Borders::ALL),
            )
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}
