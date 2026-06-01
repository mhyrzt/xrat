use super::{ConfirmKind, TuiAction, TuiApp, TuiConfigCommand, TuiView};

impl TuiApp {
    pub fn pending_confirm_command(&self) -> Option<TuiConfigCommand> {
        self.confirm.as_ref().map(|confirm| match confirm.kind {
            ConfirmKind::SoftDeleteConfig(id) => TuiConfigCommand::SoftDelete(id),
            ConfirmKind::PurgeConfig(id) => TuiConfigCommand::Purge(id),
        })
    }

    pub fn config_command_for_action(&self, action: TuiAction) -> Option<TuiConfigCommand> {
        if self.active_view != TuiView::Configs
            || self.config_list.editing_search
            || self.confirm.is_some()
        {
            return None;
        }

        let config = self.focused_config()?;
        match action {
            TuiAction::SelectFocused if !config.is_deleted => {
                Some(TuiConfigCommand::Select(config.id))
            }
            TuiAction::EnableFocused if !config.is_deleted => {
                Some(TuiConfigCommand::Enable(config.id))
            }
            TuiAction::DisableFocused if !config.is_deleted => {
                Some(TuiConfigCommand::Disable(config.id))
            }
            TuiAction::RestoreFocused if config.is_deleted => {
                Some(TuiConfigCommand::Restore(config.id))
            }
            _ => None,
        }
    }
}
