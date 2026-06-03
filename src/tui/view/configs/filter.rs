use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::{ConfigFilter, TuiApp};
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
    let filter_label = match app.config_list.filter {
        ConfigFilter::None => String::new(),
        f => format!("  [{}]", f.label()),
    };
    let proto_label = match &app.config_list.protocol_filter {
        None => String::new(),
        Some(p) => format!("  [proto:{p}]"),
    };
    let mut spans = vec![
        Span::styled("Search: ", theme::muted_style()),
        Span::raw(search),
        Span::raw("   "),
        Span::styled("Sort: ", theme::muted_style()),
        Span::raw(app.config_list.sort.label()),
    ];
    if !filter_label.is_empty() {
        spans.push(Span::styled(filter_label, theme::accent_style()));
    }
    if !proto_label.is_empty() {
        spans.push(Span::styled(proto_label, theme::accent_style()));
    }
    spans.extend([
        Span::raw("   "),
        Span::styled("Visible: ", theme::muted_style()),
        Span::raw(app.visible_configs().len().to_string()),
        Span::raw("   [/]search  [f]deleted  [F]ilter  [P]roto  [s]ort"),
    ]);

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .block(Block::default().title(" Filter ").borders(Borders::ALL)),
        area,
    );
}
