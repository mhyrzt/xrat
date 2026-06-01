use super::{ConfirmKind, ConfirmState, TuiApp, TuiView};

impl TuiApp {
    pub(super) fn push_search_char(&mut self, ch: char) {
        if !self.config_list.editing_search || ch.is_control() {
            return;
        }

        self.config_list.search_query.push(ch);
        self.config_list.focused = 0;
        self.status_message = format!("{} visible configs", self.visible_config_indices().len());
    }

    pub(super) fn pop_search_char(&mut self) {
        if !self.config_list.editing_search {
            return;
        }

        self.config_list.search_query.pop();
        self.config_list.focused = 0;
        self.status_message = format!("{} visible configs", self.visible_config_indices().len());
    }

    pub(super) fn clear_search(&mut self) {
        self.config_list.search_query.clear();
        self.config_list.focused = 0;
        self.status_message = "search cleared".to_string();
    }

    pub(super) fn close_search(&mut self) {
        self.config_list.editing_search = false;
        self.status_message = self.config_filter_summary();
    }

    pub(super) fn cycle_config_sort(&mut self) {
        if self.active_view != TuiView::Configs || self.config_list.editing_search {
            return;
        }

        self.config_list.sort = self.config_list.sort.next();
        self.config_list.focused = 0;
        self.status_message = format!("sort: {}", self.config_list.sort.label());
    }

    pub(super) fn toggle_deleted_filter(&mut self) {
        if self.active_view != TuiView::Configs || self.config_list.editing_search {
            return;
        }

        self.config_list.include_deleted = !self.config_list.include_deleted;
        self.config_list.focused = 0;
        self.status_message = if self.config_list.include_deleted {
            "showing deleted configs".to_string()
        } else {
            "hiding deleted configs".to_string()
        };
    }

    pub(super) fn request_delete_focused(&mut self) {
        if self.active_view != TuiView::Configs
            || self.config_list.editing_search
            || self.confirm.is_some()
        {
            return;
        }

        let Some(config) = self.focused_config() else {
            return;
        };
        if config.is_deleted {
            self.status_message = "config is already deleted".to_string();
            return;
        }

        self.confirm = Some(ConfirmState {
            kind: ConfirmKind::SoftDeleteConfig(config.id),
            title: " Soft delete config ".to_string(),
            message: format!(
                "Soft delete #{} {}? The row will be hidden unless deleted configs are shown.",
                config.id,
                config.display_name()
            ),
        });
        self.status_message = "confirm soft delete".to_string();
    }

    pub(super) fn request_purge_focused(&mut self) {
        if self.active_view != TuiView::Configs
            || self.config_list.editing_search
            || self.confirm.is_some()
        {
            return;
        }

        let Some(config) = self.focused_config() else {
            return;
        };
        self.confirm = Some(ConfirmState {
            kind: ConfirmKind::PurgeConfig(config.id),
            title: " Purge config ".to_string(),
            message: format!(
                "Permanently delete #{} {}? This cannot be undone.",
                config.id,
                config.display_name()
            ),
        });
        self.status_message = "confirm purge".to_string();
    }

    pub(super) fn move_focus(&mut self, delta: isize) {
        match self.active_view {
            TuiView::Configs => self.move_config_focus(delta),
            TuiView::Sources => self.move_source_focus(delta),
            TuiView::Tests | TuiView::Runtime => {}
        }
    }

    pub(super) fn clamp_config_focus(&mut self) {
        let len = self.visible_config_indices().len();
        if len == 0 {
            self.config_list.focused = 0;
        } else if self.config_list.focused >= len {
            self.config_list.focused = len - 1;
        }
    }

    pub(super) fn clamp_source_focus(&mut self) {
        let len = self.data.sources.len();
        if len == 0 {
            self.source_list.focused = 0;
        } else if self.source_list.focused >= len {
            self.source_list.focused = len - 1;
        }
    }

    pub(super) fn visible_config_indices(&self) -> Vec<usize> {
        let query = self.config_list.search_query.trim().to_lowercase();
        let mut indices: Vec<usize> = self
            .data
            .configs
            .iter()
            .enumerate()
            .filter_map(|(idx, config)| {
                if query.is_empty() || config.matches_search(&query) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        indices.sort_by(|left, right| {
            let left = &self.data.configs[*left];
            let right = &self.data.configs[*right];
            self.config_list.sort.compare(left, right)
        });
        indices
    }

    fn move_config_focus(&mut self, delta: isize) {
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
        if let Some(config) = self.focused_config() {
            self.status_message = format!("#{} {}", config.id, config.display_name());
        }
    }

    fn move_source_focus(&mut self, delta: isize) {
        let len = self.data.sources.len();
        if len == 0 {
            self.source_list.focused = 0;
            return;
        }

        let next = if delta.is_negative() {
            self.source_list
                .focused
                .saturating_sub(delta.unsigned_abs())
        } else {
            (self.source_list.focused + delta as usize).min(len - 1)
        };

        self.source_list.focused = next;
        if let Some(source) = self.focused_source() {
            self.status_message = format!("source #{} {}", source.id, source.display_name());
        }
    }
}
