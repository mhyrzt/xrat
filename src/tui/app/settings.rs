use std::collections::BTreeSet;

use super::{SettingsEditState, SettingsModalState, SettingsPane, TuiApp};
use crate::app::config::{SettingKind, SettingValue};

impl SettingsModalState {
    pub(crate) fn sections(&self) -> Vec<String> {
        self.session
            .sections(&self.query)
            .into_iter()
            .map(|section| section.split('.').take(2).collect::<Vec<_>>().join("."))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(crate) fn selected_section(&self) -> Option<String> {
        self.sections().get(self.section_index).cloned()
    }

    pub(crate) fn visible_setting_indices(&self) -> Vec<usize> {
        let Some(section) = self.selected_section() else {
            return Vec::new();
        };
        let include_descendants = section.contains('.');
        let child_prefix = format!("{section}.");
        let query = self.query.trim().to_ascii_lowercase();
        self.session
            .settings
            .iter()
            .enumerate()
            .filter(|(_, setting)| {
                (setting.section == section
                    || (include_descendants && setting.section.starts_with(&child_prefix)))
                    && (query.is_empty()
                        || setting.path.to_ascii_lowercase().contains(&query)
                        || setting.label.to_ascii_lowercase().contains(&query))
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub(crate) fn selected_setting_index(&self) -> Option<usize> {
        self.visible_setting_indices()
            .get(self.field_index)
            .copied()
    }

    fn clamp_selection(&mut self) {
        let sections = self.sections();
        self.section_index = self.section_index.min(sections.len().saturating_sub(1));
        let fields = self.visible_setting_indices();
        self.field_index = self.field_index.min(fields.len().saturating_sub(1));
    }
}

impl TuiApp {
    pub(crate) fn append_settings_text(&mut self, text: &str) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        if let Some(editing) = &mut modal.editing {
            editing.input.push_str(text);
        } else if modal.searching {
            modal.query.push_str(text);
            modal.section_index = 0;
            modal.field_index = 0;
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn settings_move(&mut self, direction: i32) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        let length = match modal.pane {
            SettingsPane::Sections => modal.sections().len(),
            SettingsPane::Fields => modal.visible_setting_indices().len(),
        };
        if length == 0 {
            return;
        }
        let current = match modal.pane {
            SettingsPane::Sections => &mut modal.section_index,
            SettingsPane::Fields => &mut modal.field_index,
        };
        if direction < 0 {
            *current = current.saturating_sub(1);
        } else {
            *current = (*current + 1).min(length - 1);
        }
        if modal.pane == SettingsPane::Sections {
            modal.field_index = 0;
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn settings_switch_pane(&mut self) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        modal.pane = match modal.pane {
            SettingsPane::Sections => SettingsPane::Fields,
            SettingsPane::Fields => SettingsPane::Sections,
        };
    }

    pub(super) fn settings_focus_pane(&mut self, pane: SettingsPane) {
        if let Some(modal) = &mut self.settings_modal {
            modal.pane = pane;
            modal.error = None;
            modal.notice = None;
        }
    }

    pub(super) fn settings_begin_search(&mut self) {
        if let Some(modal) = &mut self.settings_modal {
            modal.searching = true;
            modal.error = None;
            modal.notice = None;
        }
    }

    pub(super) fn settings_input(&mut self, ch: char) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        if let Some(editing) = &mut modal.editing {
            editing.input.push(ch);
        } else if modal.searching {
            modal.query.push(ch);
            modal.section_index = 0;
            modal.field_index = 0;
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn settings_backspace(&mut self) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        if let Some(editing) = &mut modal.editing {
            editing.input.pop();
        } else if modal.searching {
            modal.query.pop();
            modal.section_index = 0;
            modal.field_index = 0;
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn settings_clear_input(&mut self) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        if let Some(editing) = &mut modal.editing {
            editing.input.clear();
        } else if modal.searching {
            modal.query.clear();
            modal.section_index = 0;
            modal.field_index = 0;
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn settings_submit(&mut self) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        if modal.searching {
            modal.searching = false;
            modal.clamp_selection();
            return;
        }
        if let Some(editing) = modal.editing.take() {
            match modal.session.settings[editing.setting_index].set_from_input(&editing.input) {
                Ok(()) => modal.error = None,
                Err(error) => {
                    modal.error = Some(error);
                    modal.editing = Some(editing);
                }
            }
            return;
        }
        if modal.pane == SettingsPane::Sections {
            modal.pane = SettingsPane::Fields;
            modal.error = None;
            return;
        }
        let Some(setting_index) = modal.selected_setting_index() else {
            return;
        };
        let setting = &mut modal.session.settings[setting_index];
        match setting.kind {
            SettingKind::Bool => {
                setting.toggle();
            }
            SettingKind::Enum(_) => {
                setting.cycle_enum(1);
            }
            SettingKind::Secret => {
                modal.editing = Some(SettingsEditState {
                    setting_index,
                    input: String::new(),
                });
            }
            _ => {
                modal.editing = Some(SettingsEditState {
                    setting_index,
                    input: setting.value.edit_text(),
                });
            }
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn settings_cycle(&mut self, direction: i32) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        if modal.pane != SettingsPane::Fields {
            return;
        }
        let Some(setting_index) = modal.selected_setting_index() else {
            return;
        };
        if !modal.session.settings[setting_index].cycle_enum(direction)
            && matches!(
                modal.session.settings[setting_index].value,
                SettingValue::Bool(_)
            )
        {
            modal.session.settings[setting_index].toggle();
        }
        modal.error = None;
        modal.notice = None;
    }

    pub(super) fn settings_reset(&mut self) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        if modal.pane != SettingsPane::Fields {
            return;
        }
        let Some(setting_index) = modal.selected_setting_index() else {
            return;
        };
        modal.session.settings[setting_index].reset_to_default();
        modal.error = None;
        modal.notice = None;
    }

    pub(crate) fn prepare_settings_save(&mut self) -> bool {
        let Some(modal) = &mut self.settings_modal else {
            return false;
        };
        if modal.searching {
            modal.searching = false;
            modal.clamp_selection();
        }
        if let Some(editing) = modal.editing.take() {
            match modal.session.settings[editing.setting_index].set_from_input(&editing.input) {
                Ok(()) => {}
                Err(error) => {
                    modal.error = Some(error);
                    modal.notice = None;
                    modal.editing = Some(editing);
                    return false;
                }
            }
        }
        modal.error = None;
        modal.notice = None;
        true
    }

    pub(super) fn settings_confirm_discard(&mut self, discard: bool) {
        let Some(modal) = &mut self.settings_modal else {
            return;
        };
        if discard {
            self.settings_modal = None;
            self.needs_full_clear = true;
        } else {
            modal.discard_confirm = false;
        }
    }
}
