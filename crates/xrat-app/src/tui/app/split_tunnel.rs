use crate::app::config::{TunSettings, TunSplitMode};
use crate::app::services::split_tunnel::{self, DiscoveredApp};

use super::TuiApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitListTab {
    #[default]
    Blacklist,
    Whitelist,
}

impl SplitListTab {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blacklist => "blacklist",
            Self::Whitelist => "whitelist",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitPane {
    #[default]
    Configured,
    Discovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitModalMode {
    Browse,
    Search,
    AddInput,
}

#[derive(Debug, Clone)]
pub struct SplitModalState {
    pub tun_enabled: bool,
    pub split_mode: TunSplitMode,
    pub blacklist: Vec<String>,
    pub whitelist: Vec<String>,
    pub original_tun_enabled: bool,
    pub original_split_mode: TunSplitMode,
    pub original_blacklist: Vec<String>,
    pub original_whitelist: Vec<String>,
    pub list_tab: SplitListTab,
    pub pane: SplitPane,
    pub configured_index: usize,
    pub discovered_index: usize,
    pub discovered_apps: Vec<DiscoveredApp>,
    pub search_query: String,
    pub searching: bool,
    pub adding_input: Option<String>,
    pub error: Option<String>,
    pub notice: Option<String>,
}

impl SplitModalState {
    pub fn from_tun(tun: &TunSettings) -> Self {
        let list_tab = match tun.split_mode {
            TunSplitMode::Whitelist => SplitListTab::Whitelist,
            TunSplitMode::All | TunSplitMode::Blacklist => SplitListTab::Blacklist,
        };
        let discovered_apps = split_tunnel::discover_apps(true, true, None);
        Self {
            tun_enabled: tun.enabled,
            split_mode: tun.split_mode,
            blacklist: tun.blacklist.clone(),
            whitelist: tun.whitelist.clone(),
            original_tun_enabled: tun.enabled,
            original_split_mode: tun.split_mode,
            original_blacklist: tun.blacklist.clone(),
            original_whitelist: tun.whitelist.clone(),
            list_tab,
            pane: SplitPane::Configured,
            configured_index: 0,
            discovered_index: 0,
            discovered_apps,
            search_query: String::new(),
            searching: false,
            adding_input: None,
            error: None,
            notice: None,
        }
    }

    pub fn mode(&self) -> SplitModalMode {
        if self.adding_input.is_some() {
            SplitModalMode::AddInput
        } else if self.searching {
            SplitModalMode::Search
        } else {
            SplitModalMode::Browse
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.tun_enabled != self.original_tun_enabled
            || self.split_mode != self.original_split_mode
            || self.blacklist != self.original_blacklist
            || self.whitelist != self.original_whitelist
    }

    pub fn active_list(&self) -> &[String] {
        match self.list_tab {
            SplitListTab::Blacklist => &self.blacklist,
            SplitListTab::Whitelist => &self.whitelist,
        }
    }

    pub fn active_list_mut(&mut self) -> &mut Vec<String> {
        match self.list_tab {
            SplitListTab::Blacklist => &mut self.blacklist,
            SplitListTab::Whitelist => &mut self.whitelist,
        }
    }

    pub fn visible_discovered_indices(&self) -> Vec<usize> {
        let q = self.search_query.trim().to_ascii_lowercase();
        self.discovered_apps
            .iter()
            .enumerate()
            .filter(|(_, app)| {
                q.is_empty()
                    || app.rule_entry.to_ascii_lowercase().contains(&q)
                    || app.name.to_ascii_lowercase().contains(&q)
                    || app
                        .exec_path
                        .as_ref()
                        .is_some_and(|p| p.to_ascii_lowercase().contains(&q))
                    || app
                        .desktop_id
                        .as_ref()
                        .is_some_and(|id| id.to_ascii_lowercase().contains(&q))
            })
            .map(|(idx, _)| idx)
            .collect()
    }

    pub fn clamp_selection(&mut self) {
        let cfg_len = self.active_list().len();
        self.configured_index = self.configured_index.min(cfg_len.saturating_sub(1));
        let disc_len = self.visible_discovered_indices().len();
        self.discovered_index = self.discovered_index.min(disc_len.saturating_sub(1));
    }
}

impl TuiApp {
    pub(super) fn split_move(&mut self, direction: i32) {
        let Some(modal) = &mut self.split_modal else {
            return;
        };
        let len = match modal.pane {
            SplitPane::Configured => modal.active_list().len(),
            SplitPane::Discovered => modal.visible_discovered_indices().len(),
        };
        if len == 0 {
            return;
        }
        let idx = match modal.pane {
            SplitPane::Configured => &mut modal.configured_index,
            SplitPane::Discovered => &mut modal.discovered_index,
        };
        if direction < 0 {
            *idx = idx.saturating_sub(1);
        } else {
            *idx = (*idx + 1).min(len - 1);
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn split_switch_pane(&mut self) {
        if let Some(modal) = &mut self.split_modal {
            modal.pane = match modal.pane {
                SplitPane::Configured => SplitPane::Discovered,
                SplitPane::Discovered => SplitPane::Configured,
            };
            modal.error = None;
            modal.notice = None;
        }
    }

    pub(super) fn split_cycle_mode(&mut self) {
        if let Some(modal) = &mut self.split_modal {
            modal.split_mode = match modal.split_mode {
                TunSplitMode::All => TunSplitMode::Blacklist,
                TunSplitMode::Blacklist => TunSplitMode::Whitelist,
                TunSplitMode::Whitelist => TunSplitMode::All,
            };
            match modal.split_mode {
                TunSplitMode::Blacklist => modal.list_tab = SplitListTab::Blacklist,
                TunSplitMode::Whitelist => modal.list_tab = SplitListTab::Whitelist,
                TunSplitMode::All => {}
            }
            modal.clamp_selection();
            modal.error = None;
            modal.notice = None;
        }
    }

    pub(super) fn split_toggle_tun(&mut self) {
        if let Some(modal) = &mut self.split_modal {
            modal.tun_enabled = !modal.tun_enabled;
            modal.error = None;
            modal.notice = None;
        }
    }

    pub(super) fn split_select_list(&mut self, tab: SplitListTab) {
        if let Some(modal) = &mut self.split_modal {
            modal.list_tab = tab;
            modal.configured_index = 0;
            modal.clamp_selection();
            modal.error = None;
            modal.notice = None;
        }
    }

    pub(super) fn split_begin_add(&mut self) {
        if let Some(modal) = &mut self.split_modal {
            modal.searching = false;
            modal.adding_input = Some(String::new());
            modal.error = None;
            modal.notice = None;
        }
    }

    pub(super) fn split_begin_search(&mut self) {
        if let Some(modal) = &mut self.split_modal {
            modal.adding_input = None;
            modal.searching = true;
            modal.pane = SplitPane::Discovered;
            modal.error = None;
            modal.notice = None;
        }
    }

    pub(super) fn split_input(&mut self, ch: char) {
        let Some(modal) = &mut self.split_modal else {
            return;
        };
        if let Some(input) = &mut modal.adding_input {
            input.push(ch);
        } else if modal.searching {
            modal.search_query.push(ch);
            modal.discovered_index = 0;
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn split_backspace(&mut self) {
        let Some(modal) = &mut self.split_modal else {
            return;
        };
        if let Some(input) = &mut modal.adding_input {
            input.pop();
        } else if modal.searching {
            modal.search_query.pop();
            modal.discovered_index = 0;
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn split_clear_input(&mut self) {
        let Some(modal) = &mut self.split_modal else {
            return;
        };
        if let Some(input) = &mut modal.adding_input {
            input.clear();
        } else if modal.searching {
            modal.search_query.clear();
            modal.discovered_index = 0;
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn split_submit(&mut self) {
        let Some(modal) = &mut self.split_modal else {
            return;
        };
        if modal.searching {
            modal.searching = false;
            modal.clamp_selection();
            return;
        }
        if let Some(raw_input) = modal.adding_input.take() {
            let trimmed = raw_input.trim();
            if trimmed.is_empty() {
                return;
            }
            match split_tunnel::normalize_app_specifier(trimmed) {
                Ok(_) => {
                    let list = modal.active_list_mut();
                    if !list.iter().any(|existing| existing == trimmed) {
                        list.push(trimmed.to_string());
                    }
                    modal.configured_index = modal.active_list().len().saturating_sub(1);
                    modal.error = None;
                }
                Err(err) => {
                    modal.error = Some(err);
                    modal.adding_input = Some(raw_input);
                }
            }
            return;
        }

        match modal.pane {
            SplitPane::Configured => {
                modal.adding_input = Some(String::new());
            }
            SplitPane::Discovered => {
                let visible = modal.visible_discovered_indices();
                let Some(&app_idx) = visible.get(modal.discovered_index) else {
                    return;
                };
                let spec = modal.discovered_apps[app_idx].rule_entry.clone();
                let list_name = modal.list_tab.as_str();
                let list = modal.active_list_mut();
                if let Some(pos) = list.iter().position(|item| item == &spec) {
                    list.remove(pos);
                    modal.notice = Some(format!("Removed `{spec}` from {list_name}."));
                } else {
                    list.push(spec.clone());
                    modal.notice = Some(format!("Added `{spec}` to {list_name}."));
                }
                modal.clamp_selection();
                modal.error = None;
            }
        }
    }

    pub(super) fn split_delete_focused(&mut self) {
        let Some(modal) = &mut self.split_modal else {
            return;
        };
        if modal.pane != SplitPane::Configured {
            return;
        }
        let idx = modal.configured_index;
        let list_name = modal.list_tab.as_str();
        let list = modal.active_list_mut();
        if idx < list.len() {
            let removed = list.remove(idx);
            modal.notice = Some(format!("Removed `{removed}` from {list_name}."));
            modal.clamp_selection();
            modal.error = None;
        }
    }

    pub(super) fn split_clear_list(&mut self) {
        let Some(modal) = &mut self.split_modal else {
            return;
        };
        let list_name = modal.list_tab.as_str();
        modal.active_list_mut().clear();
        modal.configured_index = 0;
        modal.notice = Some(format!("Cleared {list_name}."));
        modal.error = None;
    }

    pub(super) fn split_refresh_apps(&mut self) {
        if let Some(modal) = &mut self.split_modal {
            modal.discovered_apps = split_tunnel::discover_apps(true, true, None);
            modal.clamp_selection();
            modal.notice = Some(format!(
                "Discovered {} running/desktop applications.",
                modal.discovered_apps.len()
            ));
            modal.error = None;
        }
    }
}
