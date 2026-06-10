use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Row, Table, TableState};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    app.panel_viewport.table.set(area.height.saturating_sub(3));
    let header = Row::new(["Ref", "Name", "Kind", "Configs", "Updated"])
        .style(theme::accent_style().add_modifier(Modifier::BOLD));

    let focused_row = app.source_list.focused;
    let pseudo_style = |row_index: usize| {
        if focused_row == row_index {
            theme::accent_style().add_modifier(Modifier::BOLD)
        } else {
            theme::muted_style()
        }
    };

    let orphan_count = app
        .data
        .configs
        .iter()
        .filter(|config| config.source_id.is_none())
        .count();

    let all_row = Row::new(vec![
        Cell::from("-"),
        Cell::from("All configs"),
        Cell::from("-"),
        Cell::from(app.data.total_configs.to_string()),
        Cell::from("-"),
    ])
    .style(pseudo_style(0));

    let orphan_row = Row::new(vec![
        Cell::from("-"),
        Cell::from("Orphans"),
        Cell::from("-"),
        Cell::from(orphan_count.to_string()),
        Cell::from("-"),
    ])
    .style(pseudo_style(1));

    let source_rows = app.data.sources.iter().enumerate().map(|(idx, source)| {
        let style = if idx + 2 == focused_row {
            theme::accent_style().add_modifier(Modifier::BOLD)
        } else {
            theme::chrome_style()
        };

        Row::new(vec![
            Cell::from(source.display_ref().to_string()),
            Cell::from(source.display_name().to_string()),
            Cell::from(source.kind.clone()),
            Cell::from(source.config_count.to_string()),
            Cell::from(source.updated_at.clone()),
        ])
        .style(style)
    });

    let rows = [all_row, orphan_row].into_iter().chain(source_rows);

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(tab_title(app))
            .borders(Borders::ALL)
            .border_style(if focused {
                theme::accent_style()
            } else {
                theme::chrome_style()
            }),
    )
    .column_spacing(1);

    let mut state = TableState::default().with_selected(Some(app.source_list.focused));
    frame.render_widget(Clear, area);
    frame.render_stateful_widget(table, area, &mut state);
}

fn tab_title(app: &TuiApp) -> Line<'static> {
    Line::from(vec![
        Span::raw(" "),
        Span::styled("1:", theme::accent_style().add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(" Configs ", theme::muted_style()),
        Span::styled(
            "[Subscriptions]",
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({}) ", app.data.sources.len()),
            theme::chrome_style(),
        ),
    ])
}
