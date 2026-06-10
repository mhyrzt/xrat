use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;

use crate::tui::app::{SourceFilter, TuiApp};
use crate::tui::data::TuiSourceRow;
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, numbered_title, push_detail, render_scroll_panel};

const LABEL_WIDTH: usize = crate::tui::theme::DETAIL_LABEL_WIDTH;
const RIGHT_PAD: u16 = crate::tui::theme::DETAIL_RIGHT_PAD;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let lines = source_detail_lines(area, app.focused_source(), app.config_list.source_filter);
    let viewport = render_scroll_panel(
        frame,
        area,
        lines,
        &app.panel_scroll.detail,
        PanelStyle {
            title: numbered_title(3, "Detail"),
            focused,
            right_pad: RIGHT_PAD,
            wrap_trim: true,
        },
    );
    app.panel_viewport.detail.set(viewport);
}

fn source_detail_lines<'a>(
    area: Rect,
    source: Option<&'a TuiSourceRow>,
    filter: SourceFilter,
) -> Vec<Line<'a>> {
    let content_width = area.width.saturating_sub(2 + RIGHT_PAD) as usize;
    match source {
        Some(source) => {
            let mut lines = vec![
                Line::styled(
                    format!("{} {}", source.display_ref(), source.display_name()),
                    theme::accent_style().add_modifier(Modifier::BOLD),
                ),
                Line::raw(""),
            ];
            push_detail(
                &mut lines,
                "ID",
                source.id.to_string(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(&mut lines, "Ref", &source.r#ref, LABEL_WIDTH, content_width);
            push_detail(&mut lines, "Kind", &source.kind, LABEL_WIDTH, content_width);
            push_detail(
                &mut lines,
                "Value",
                source.value_label(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Configs",
                source.config_count.to_string(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Created",
                &source.created_at,
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Updated",
                &source.updated_at,
                LABEL_WIDTH,
                content_width,
            );
            lines
        }
        None => {
            let title = match filter {
                SourceFilter::Orphans => "Orphans",
                _ => "All configs",
            };
            vec![Line::styled(
                title,
                theme::accent_style().add_modifier(Modifier::BOLD),
            )]
        }
    }
}
