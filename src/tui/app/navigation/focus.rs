use super::TuiApp;

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
        let len = self.data.sources.len();
        if len == 0 {
            self.source_list.focused = 0;
        } else if self.source_list.focused >= len {
            self.source_list.focused = len - 1;
        }
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
        if let Some(config) = self.focused_config() {
            self.status_message = format!("#{} {}", config.id, config.display_name());
        }
    }

    pub(crate) fn move_source_focus(&mut self, delta: isize) {
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
