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

    pub(super) fn page_focus(&mut self, direction: isize) {
        let page = self.focused_viewport_rows().max(1) as isize;
        self.move_focus(direction * page);
    }

    pub(super) fn move_to_top(&mut self) {
        match self.focused_panel {
            TuiPanel::Table => match self.active_view {
                TuiView::Configs => self.move_config_focus(isize::MIN),
                TuiView::Sources => self.move_source_focus(isize::MIN),
            },
            TuiPanel::Detail => self.panel_scroll.detail.set(0),
            TuiPanel::Log => self.panel_scroll.log.set(0),
            TuiPanel::Runtime => self.panel_scroll.runtime.set(0),
        }
    }

    pub(super) fn move_to_bottom(&mut self) {
        match self.focused_panel {
            TuiPanel::Table => match self.active_view {
                TuiView::Configs => self.move_config_focus(isize::MAX),
                TuiView::Sources => self.move_source_focus(isize::MAX),
            },
            TuiPanel::Detail => self.panel_scroll.detail.set(u16::MAX),
            TuiPanel::Log => self.panel_scroll.log.set(u16::MAX),
            TuiPanel::Runtime => self.panel_scroll.runtime.set(u16::MAX),
        }
    }

    fn focused_viewport_rows(&self) -> u16 {
        match self.focused_panel {
            TuiPanel::Table => self.panel_viewport.table.get(),
            TuiPanel::Detail => self.panel_viewport.detail.get(),
            TuiPanel::Log => self.panel_viewport.log.get(),
            TuiPanel::Runtime => self.panel_viewport.runtime.get(),
        }
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
