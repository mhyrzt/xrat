mod confirm;
mod focus;
mod search;
mod sort;

use super::TuiApp;
use crate::tui::app::{TuiPanel, TuiView};

impl TuiApp {
    pub(super) fn move_focus(&mut self, delta: isize) {
        match self.focused_panel {
            TuiPanel::Table => match self.active_view {
                TuiView::Configs => self.move_config_focus(delta),
                TuiView::Sources => self.move_source_focus(delta),
            },
            TuiPanel::Detail => scroll(&self.panel_scroll.detail, delta),
            TuiPanel::Log => scroll(&self.panel_scroll.log, delta),
            TuiPanel::Runtime => scroll(&self.panel_scroll.runtime, delta),
        }
    }

    pub(super) fn focus_panel(&mut self, panel: TuiPanel) {
        self.focused_panel = panel;
    }
}

/// Adjust a scroll offset by `delta` rows, clamped at the top. The render pass
/// clamps the bottom against the live content/viewport sizes.
fn scroll(offset: &std::cell::Cell<u16>, delta: isize) {
    let current = offset.get();
    let next = if delta.is_negative() {
        current.saturating_sub(delta.unsigned_abs() as u16)
    } else {
        current.saturating_add(delta as u16)
    };
    offset.set(next);
}
