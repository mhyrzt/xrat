use ratatui::layout::Rect;
use ratatui::text::{Line, Span};

use crate::tui::theme;

pub fn push_detail<'a>(
    lines: &mut Vec<Line<'a>>,
    label: &str,
    value: impl Into<String>,
    label_width: usize,
    content_width: usize,
) {
    let value = value.into();
    let label_text = format!("{label}:");

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

pub fn append_bottom_lines(
    lines: &mut Vec<Line<'static>>,
    bottom_lines: Vec<Line<'static>>,
    area: Rect,
    border_height: u16,
) {
    let content_height = area.height.saturating_sub(border_height) as usize;
    let blank_lines = content_height.saturating_sub(lines.len() + bottom_lines.len());
    lines.extend(std::iter::repeat_with(Line::default).take(blank_lines));
    lines.extend(bottom_lines);
}
