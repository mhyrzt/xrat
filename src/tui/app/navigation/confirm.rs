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
            title: " Delete source ".to_string(),
            message: format!(
                "Delete source #{id} \"{name}\" and all its configs? Cannot be undone."
            ),
        });
        self.status_message = "confirm delete source".to_string();
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
            title: " Purge config ".to_string(),
            message: format!(
                "Permanently delete #{} {}? This cannot be undone.",
                config.id,
                config.display_name()
            ),
        });
        self.status_message = "confirm purge".to_string();
    }
}
