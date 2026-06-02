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

    pub(crate) fn cycle_config_filter(&mut self) {
        if self.active_view != TuiView::Configs || self.config_list.editing_search {
            return;
        }

        self.config_list.filter = self.config_list.filter.next();
        self.config_list.focused = 0;
        self.status_message = format!("filter: {}", self.config_list.filter.label());
    }

    pub(crate) fn cycle_protocol_filter(&mut self) {
        if self.active_view != TuiView::Configs || self.config_list.editing_search {
            return;
        }

        let mut protocols: Vec<String> = {
            let mut seen = std::collections::BTreeSet::new();
            self.data
                .configs
                .iter()
                .filter(|c| seen.insert(c.protocol.clone()))
                .map(|c| c.protocol.clone())
                .collect()
        };
        protocols.sort();

        let next = match &self.config_list.protocol_filter {
            None => protocols.into_iter().next(),
            Some(current) => {
                let pos = protocols.iter().position(|p| p == current);
                match pos {
                    Some(i) if i + 1 < protocols.len() => Some(protocols[i + 1].clone()),
                    _ => None,
                }
            }
        };

        self.config_list.protocol_filter = next;
        self.config_list.focused = 0;
        self.status_message = match &self.config_list.protocol_filter {
            Some(p) => format!("protocol: {p}"),
            None => "protocol: all".to_string(),
        };
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
        let filter = self.config_list.filter;
        let proto = self.config_list.protocol_filter.as_deref();
        let mut indices: Vec<usize> = self
            .data
            .configs
            .iter()
            .enumerate()
            .filter_map(|(idx, config)| {
                if !query.is_empty() && !config.matches_search(&query) {
                    return None;
                }
                if !filter.matches(config) {
                    return None;
                }
                if let Some(p) = proto
                    && config.protocol != p
                {
                    return None;
                }
                Some(idx)
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
