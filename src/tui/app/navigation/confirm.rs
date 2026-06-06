use super::TuiApp;
use crate::tui::app::{ConfirmKind, ConfirmState, TuiView};

impl TuiApp {
    pub(crate) fn request_delete_source(&mut self) {
        if self.active_view != TuiView::Sources || self.confirm.is_some() {
            return;
        }
        let Some(source) = self.focused_source() else {
            return;
        };
        let id = source.id;
        let name = source.display_name().to_string();
        self.confirm = Some(ConfirmState {
            kind: ConfirmKind::DeleteSource(id),
            prompt: format!(
                "delete source {} \"{name}\" + its configs?",
                source.display_ref()
            ),
        });
    }
}

impl TuiApp {
    pub(crate) fn request_delete_focused(&mut self) {
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
            return;
        }

        self.confirm = Some(ConfirmState {
            kind: ConfirmKind::SoftDeleteConfig(config.id),
            prompt: format!(
                "soft-delete {} {}?",
                config.display_ref(),
                config.display_name()
            ),
        });
    }

    /// Arm an inline confirm for clearing all persisted events from the
    /// database. This is the destructive DB clear, distinct from a view-only
    /// buffer clear.
    pub(crate) fn request_clear_events(&mut self) {
        if self.active_view != TuiView::Configs
            || self.config_list.editing_search
            || self.confirm.is_some()
        {
            return;
        }
        self.confirm = Some(ConfirmState {
            kind: ConfirmKind::ClearEvents,
            prompt: "clear all persisted events from db (cannot undo)?".to_string(),
        });
    }

    pub(crate) fn request_purge_focused(&mut self) {
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
            prompt: format!(
                "purge {} {} (cannot undo)?",
                config.display_ref(),
                config.display_name()
            ),
        });
    }
}
