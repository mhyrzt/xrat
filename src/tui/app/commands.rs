use super::{BulkOp, ConfirmKind, TuiAction, TuiApp, TuiConfigCommand, TuiView};

impl TuiApp {
    pub fn pending_confirm_command(&self) -> Option<TuiConfigCommand> {
        let confirm = self.confirm.as_ref()?;
        match confirm.kind {
            ConfirmKind::SoftDeleteConfig(id) => Some(TuiConfigCommand::SoftDelete(id)),
            ConfirmKind::PurgeConfig(id) => Some(TuiConfigCommand::Purge(id)),
            ConfirmKind::DeleteSource(_)
            | ConfirmKind::ClearEvents
            | ConfirmKind::RestartAfterSettings => None,
        }
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

    pub fn test_config_ids(&self) -> Vec<i64> {
        match self.test_state.scope {
            super::TestScope::Focused => self
                .focused_config()
                .map(|row| row.id)
                .into_iter()
                .collect(),
            super::TestScope::Filtered => self
                .visible_configs()
                .into_iter()
                .filter(|config| config.is_enabled && !config.is_deleted)
                .map(|config| config.id)
                .collect(),
            super::TestScope::AllEnabled => self
                .data
                .configs
                .iter()
                .filter(|config| config.is_enabled && !config.is_deleted)
                .map(|config| config.id)
                .collect(),
            super::TestScope::Failed => self
                .data
                .configs
                .iter()
                .filter(|config| {
                    config.failure_reason.is_some() && config.is_enabled && !config.is_deleted
                })
                .map(|config| config.id)
                .collect(),
            super::TestScope::Stale => self
                .data
                .configs
                .iter()
                .filter(|config| {
                    config.real_delay_ms.is_none() && config.tcp_ms.is_none() && !config.is_deleted
                })
                .map(|config| config.id)
                .collect(),
        }
    }

    /// Config ids targeted by a bulk operation, resolved against current data
    /// and the active filter/search.
    pub fn bulk_config_ids(&self, op: BulkOp) -> Vec<i64> {
        match op {
            BulkOp::DeleteFailed | BulkOp::PurgeFailed => self
                .data
                .configs
                .iter()
                .filter(|config| {
                    config.failure_reason.is_some() && config.is_enabled && !config.is_deleted
                })
                .map(|config| config.id)
                .collect(),
            BulkOp::DeleteFiltered => self
                .visible_configs()
                .into_iter()
                .filter(|config| config.is_enabled && !config.is_deleted)
                .map(|config| config.id)
                .collect(),
            BulkOp::DeleteDisabled => self
                .data
                .configs
                .iter()
                .filter(|config| !config.is_enabled && !config.is_deleted)
                .map(|config| config.id)
                .collect(),
            BulkOp::PurgeFilteredDeleted | BulkOp::RestoreFilteredDeleted => self
                .visible_configs()
                .into_iter()
                .filter(|config| config.is_deleted)
                .map(|config| config.id)
                .collect(),
            BulkOp::PurgeAllDeleted | BulkOp::RestoreAllDeleted => self
                .data
                .configs
                .iter()
                .filter(|config| config.is_deleted)
                .map(|config| config.id)
                .collect(),
        }
    }

    /// Arm an inline confirm for a bulk op, or report when nothing matches.
    pub fn request_bulk(&mut self, op: BulkOp) {
        if self.active_view != TuiView::Configs {
            return;
        }
        let count = self.bulk_config_ids(op).len();
        if count == 0 {
            self.pending_bulk = None;
            return;
        }
        self.pending_bulk = Some(op);
    }
}
