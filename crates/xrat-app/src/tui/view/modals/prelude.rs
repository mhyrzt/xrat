pub(crate) use ratatui::Frame;
pub(crate) use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
pub(crate) use ratatui::text::{Line, Span};
pub(crate) use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
pub(crate) use unicode_width::UnicodeWidthStr;

pub(crate) use crate::app::config::{EditableSetting, SettingKind, SettingValue};
pub(crate) use crate::tui::app::{
    ImportModalStep, SettingsModalState, SettingsMode, SettingsPane, TuiApp,
};

/// Centered rectangle of a fixed size, clamped to the available area.
pub(crate) fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
