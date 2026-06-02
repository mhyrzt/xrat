mod confirm;
mod focus;
mod search;
mod sort;

use super::TuiApp;
use crate::tui::app::TuiView;

impl TuiApp {
    pub(super) fn move_focus(&mut self, delta: isize) {
        match self.active_view {
            TuiView::Configs => self.move_config_focus(delta),
            TuiView::Sources => self.move_source_focus(delta),
            TuiView::Tests | TuiView::Runtime | TuiView::Diagnostics => {}
        }
    }
}
