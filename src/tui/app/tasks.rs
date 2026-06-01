use super::TuiApp;

impl TuiApp {
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
                self.status_message = message;
            }
            crate::tui::task::TuiTaskEvent::Failed { kind, error } => {
                self.task_state
                    .apply(&crate::tui::task::TuiTaskEvent::Failed {
                        kind,
                        error: error.clone(),
                    });
                self.status_message = format!("operation failed: {error}");
            }
            event => {
                self.task_state.apply(&event);
                self.status_message = self.task_state.label();
            }
        }
    }
}
