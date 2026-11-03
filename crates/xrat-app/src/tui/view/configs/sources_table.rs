use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Table, TableState,
};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    app.panel_viewport.table.set(area.height.saturating_sub(3));
    let header = Row::new(["Ref", "Name", "Kind", "Configs", "Updated"])
        .style(theme::secondary_style().add_modifier(Modifier::BOLD));

    let focused_row = app.source_list.focused;
    let pseudo_style = |row_index: usize| {
        if focused_row == row_index {
            theme::selection_style().add_modifier(Modifier::BOLD)
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
        let is_focused = idx + 2 == focused_row;
        let style = if is_focused {
            theme::selection_style().add_modifier(Modifier::BOLD)
        } else {
            theme::chrome_style()
        };

        Row::new(vec![
            Cell::from(source.display_ref().to_string()).style(theme::secondary_style()),
            Cell::from(source.display_name().to_string()),
            Cell::from(source.kind.clone()),
            Cell::from(source.config_count.to_string()),
            Cell::from(source.updated_at.clone()),
        ])
        .style(style)
    });

    let total_rows = 2 + app.data.sources.len();
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
    .highlight_symbol("▶ ")
    .highlight_spacing(HighlightSpacing::Always)
    .block(
        Block::default()
            .title(tab_title(app))
            .borders(Borders::ALL)
            .border_type(theme::border_type())
            .border_style(theme::border_style(focused)),
    )
    .column_spacing(1);

    let mut state = TableState::default()
        .with_offset(app.source_list.table_offset.get())
        .with_selected(Some(app.source_list.focused));
    crate::tui::theme::clear(frame, area);
    frame.render_stateful_widget(table, area, &mut state);
    app.source_list.table_offset.set(state.offset());

    let viewport = area.height.saturating_sub(3) as usize;
    if total_rows > viewport {
        let mut scrollbar_state = ScrollbarState::new(total_rows)
            .viewport_content_length(viewport)
            .position(app.source_list.focused);
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

fn tab_title(app: &TuiApp) -> Line<'static> {
    Line::from(vec![
        Span::raw(" "),
        Span::styled("1:", theme::secondary_style().add_modifier(Modifier::BOLD)),
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
