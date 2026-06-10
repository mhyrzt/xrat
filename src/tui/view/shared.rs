use std::cell::Cell;

use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Wrap,
};

use crate::tui::theme;

pub fn push_detail<'a>(
    lines: &mut Vec<Line<'a>>,
    label: &str,
    value: impl Into<String>,
    label_width: usize,
    content_width: usize,
) {
    let value = value.into();
    let label_text = format!("{label} ");

    if label_width + value.chars().count() <= content_width {
        lines.push(Line::from(vec![
            Span::styled(format!("{label_text:<label_width$}"), theme::muted_style()),
            Span::raw(value),
        ]));
    } else {
        lines.push(Line::from(Span::styled(label_text, theme::muted_style())));
        lines.push(Line::from(Span::raw(format!("  {value}"))));
    }
}

pub struct PanelStyle {
    pub title: Line<'static>,
    pub focused: bool,
    pub right_pad: u16,
    pub wrap_trim: bool,
}

pub fn numbered_title(number: u8, title: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw(" "),
        Span::styled(
            format!("{number}:"),
            theme::accent_style().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" {title} ")),
    ])
}

/// Render a bordered, vertically scrollable card. The focused card gets an
/// accent border. `scroll` is clamped here against the live content and
/// viewport heights, and a scrollbar is drawn when the content overflows.
pub fn render_scroll_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    lines: Vec<Line<'_>>,
    scroll: &Cell<u16>,
    style: PanelStyle,
) {
    let border_style = if style.focused {
        theme::accent_style()
    } else {
        theme::chrome_style()
    };
    let block = Block::default()
        .title(style.title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .padding(Padding::new(0, style.right_pad, 0, 0));

    let viewport = block.inner(area).height;
    let content_len = lines.len() as u16;
    let offset = scroll.get().min(content_len.saturating_sub(viewport));
    scroll.set(offset);

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .style(theme::chrome_style())
            .wrap(Wrap {
                trim: style.wrap_trim,
            })
            .scroll((offset, 0)),
        area,
    );

    if content_len > viewport {
        let mut state = ScrollbarState::new(content_len as usize)
            .viewport_content_length(viewport as usize)
            .position(offset as usize);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin::new(0, 1)),
            &mut state,
        );
    }
}
