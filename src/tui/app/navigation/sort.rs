use super::TuiApp;
use crate::tui::app::TuiView;

impl TuiApp {
    pub(crate) fn clear_search(&mut self) {
        self.config_list.search_query.clear();
        self.config_list.focused = 0;
        self.status_message = "search cleared".to_string();
    }

    pub(crate) fn cycle_config_sort(&mut self) {
        if self.active_view != TuiView::Configs || self.config_list.editing_search {
            return;
        }

        self.config_list.sort = self.config_list.sort.next();
        self.config_list.focused = 0;
        self.status_message = format!("sort: {}", self.config_list.sort.label());
    }

    pub(crate) fn toggle_deleted_filter(&mut self) {
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

    pub(crate) fn visible_config_indices(&self) -> Vec<usize> {
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
}
