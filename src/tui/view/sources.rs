use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::data::TuiSourceRow;
use crate::tui::theme;
use crate::tui::view::shared::detail_line;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_table(frame, columns[0], app);
    render_detail(frame, columns[1], app.focused_source());
}

fn render_table(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let header = Row::new(["ID", "Name", "Kind", "Configs", "Updated"])
        .style(theme::accent_style().add_modifier(Modifier::BOLD));

    let rows = app.data.sources.iter().enumerate().map(|(idx, source)| {
        let style = if idx == app.source_list.focused {
            theme::accent_style().add_modifier(Modifier::BOLD)
        } else {
            theme::chrome_style()
        };

        Row::new(vec![
            Cell::from(source.id.to_string()),
            Cell::from(source.display_name().to_string()),
            Cell::from(source.kind.clone()),
            Cell::from(source.config_count.to_string()),
            Cell::from(source.updated_at.clone()),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Sources ({}) ", app.data.sources.len()))
            .borders(Borders::ALL),
    )
    .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, source: Option<&TuiSourceRow>) {
    let lines = match source {
        Some(source) => vec![
            Line::styled(
                format!("#{} {}", source.id, source.display_name()),
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            detail_line("Kind", &source.kind),
            detail_line("Value", source.value_label()),
            detail_line("Configs", source.config_count.to_string()),
            detail_line("Created", &source.created_at),
            detail_line("Updated", &source.updated_at),
            Line::raw(""),
            Line::styled("Actions", theme::muted_style()),
            Line::raw("r refresh focused  R refresh all  i import"),
        ],
        None => vec![
            Line::styled(
                "No sources",
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::raw("Import a subscription with `xrat import <input>`."),
        ],
    };

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Source Detail ")
                    .borders(Borders::ALL),
            )
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}
