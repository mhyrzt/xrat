use super::TuiApp;

const MAX_LOG_ENTRIES: usize = 500;

impl TuiApp {
    pub fn push_log(&mut self, message: impl Into<String>) {
        let msg = message.into();
        if self.event_log.len() >= MAX_LOG_ENTRIES {
            self.event_log.remove(0);
        }
        self.event_log.push(msg);
    }

    pub fn apply_task_event(&mut self, event: crate::tui::task::TuiTaskEvent) {
        match event {
            crate::tui::task::TuiTaskEvent::Completed {
                kind,
                message,
                data,
            } => {
                self.task_state
                    .apply(&crate::tui::task::TuiTaskEvent::Completed {
                        kind,
                        message: message.clone(),
                        data: None,
                    });
                if let Some(data) = data {
                    self.reload_data(data);
                }
                if kind == crate::tui::task::TuiTaskKind::TestBatch {
                    self.testing_config_ids.clear();
                }
                self.push_log(format!("OK  {message}"));
            }
            crate::tui::task::TuiTaskEvent::Failed { kind, error, data } => {
                self.task_state
                    .apply(&crate::tui::task::TuiTaskEvent::Failed {
                        kind,
                        error: error.clone(),
                        data: None,
                    });
                if let Some(data) = data {
                    self.reload_data(data);
                }
                if kind == crate::tui::task::TuiTaskKind::TestBatch {
                    self.testing_config_ids.clear();
                }
                self.push_log(format!("ERR {error}"));
            }
            crate::tui::task::TuiTaskEvent::Cancelled { kind } => {
                self.task_state
                    .apply(&crate::tui::task::TuiTaskEvent::Cancelled { kind });
                if kind == crate::tui::task::TuiTaskKind::TestBatch {
                    self.testing_config_ids.clear();
                }
                self.push_log(format!("OK  {:?} cancelled", kind));
            }
            crate::tui::task::TuiTaskEvent::ConfigTested { row, done, total } => {
                self.task_state
                    .apply(&crate::tui::task::TuiTaskEvent::ConfigTested {
                        row: row.clone(),
                        done,
                        total,
                    });
                self.testing_config_ids.retain(|id| *id != row.id);
                self.replace_config_row(row);
            }
            event => {
                self.task_state.apply(&event);
            }
        }
    }
}
