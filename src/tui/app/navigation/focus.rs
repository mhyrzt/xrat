use super::TuiApp;
use crate::tui::app::SourceFilter;

/// Sources tab rows: 0 = All, 1 = Orphans, 2.. = concrete sources.
const SOURCE_PSEUDO_ROWS: usize = 2;

impl TuiApp {
    pub(crate) fn clamp_config_focus(&mut self) {
        let len = self.visible_config_indices().len();
        if len == 0 {
            self.config_list.focused = 0;
        } else if self.config_list.focused >= len {
            self.config_list.focused = len - 1;
        }
    }

    pub(crate) fn clamp_source_focus(&mut self) {
        let len = self.data.sources.len() + SOURCE_PSEUDO_ROWS;
        if self.source_list.focused >= len {
            self.source_list.focused = len - 1;
        }
        self.sync_source_filter();
    }

    /// Mirror the focused Sources-tab row onto the configs source filter.
    pub(crate) fn sync_source_filter(&mut self) {
        self.config_list.source_filter = match self.source_list.focused {
            0 => SourceFilter::All,
            1 => SourceFilter::Orphans,
            _ => self
                .focused_source()
                .map(|source| SourceFilter::Source(source.id))
                .unwrap_or(SourceFilter::All),
        };
    }

    pub(crate) fn move_config_focus(&mut self, delta: isize) {
        if self.config_list.editing_search {
            return;
        }

        let len = self.visible_config_indices().len();
        if len == 0 {
            self.config_list.focused = 0;
            return;
        }

        let next = if delta.is_negative() {
            self.config_list
                .focused
                .saturating_sub(delta.unsigned_abs())
        } else {
            (self.config_list.focused + delta as usize).min(len - 1)
        };

        self.config_list.focused = next;
        self.panel_scroll.detail.set(0);
    }

    pub(crate) fn move_source_focus(&mut self, delta: isize) {
        let len = self.data.sources.len() + SOURCE_PSEUDO_ROWS;

        let next = if delta.is_negative() {
            self.source_list
                .focused
                .saturating_sub(delta.unsigned_abs())
        } else {
            (self.source_list.focused + delta as usize).min(len - 1)
        };

        self.source_list.focused = next;
        self.panel_scroll.detail.set(0);
        self.sync_source_filter();
    }
}
