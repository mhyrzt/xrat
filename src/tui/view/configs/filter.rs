use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let cursor = if app.config_list.editing_search {
        "_"
    } else {
        ""
    };
    let search = if app.config_list.search_query.is_empty() {
        "<none>".to_string()
    } else {
        format!("{}{}", app.config_list.search_query, cursor)
    };
    let text = Line::from(vec![
        Span::styled("Search: ", theme::muted_style()),
        Span::raw(search),
        Span::raw("   "),
        Span::styled("Sort: ", theme::muted_style()),
        Span::raw(app.config_list.sort.label()),
        Span::raw("   "),
        Span::styled("Visible: ", theme::muted_style()),
        Span::raw(app.visible_configs().len().to_string()),
        Span::raw("   / search  f deleted  s sort"),
    ]);

    frame.render_widget(
        Paragraph::new(text).block(Block::default().title(" Filter ").borders(Borders::ALL)),
        area,
    );
}
