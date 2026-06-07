use crate::tui::cancel::{TuiCancellationReceiver, TuiCancellationToken, new_cancellation};

use super::{TuiTaskEvent, TuiTaskKind};

#[derive(Debug, Default)]
pub struct TuiTaskState {
    pub running: Option<TuiTaskKind>,
    pub last_summary: Option<String>,
    pub last_error: Option<String>,
    pub completed_count: usize,
    pub cancellation: Option<TuiCancellationToken>,
    pub progress_done: usize,
    pub progress_total: usize,
}

impl TuiTaskState {
    pub fn label(&self) -> String {
        match self.running {
            Some(kind) if self.cancellation.as_ref().is_some_and(|t| t.is_cancelled()) => {
                format!("{:?} cancelling", kind)
            }
            Some(TuiTaskKind::SourceRefresh) => "Subscriptions refreshing".to_string(),
            Some(kind) => format!("{:?} running", kind),
            None => self
                .last_summary
                .clone()
                .or_else(|| self.last_error.clone())
                .unwrap_or_else(|| "idle".to_string()),
        }
    }

    pub fn start(&mut self, kind: TuiTaskKind) -> (TuiCancellationToken, TuiCancellationReceiver) {
        self.running = Some(kind);
        self.last_error = None;
        self.progress_done = 0;
        self.progress_total = 0;
        let (token, receiver) = new_cancellation();
        self.cancellation = Some(token.clone());
        (token, receiver)
    }

    pub fn cancel(&mut self) -> bool {
        let Some(token) = self.cancellation.as_ref() else {
            return false;
        };
        token.cancel();
        true
    }

    pub fn apply(&mut self, event: &TuiTaskEvent) {
        match event {
            TuiTaskEvent::Started { kind } => {
                self.running = Some(*kind);
                self.last_error = None;
            }
            TuiTaskEvent::Progress { done, total, .. } => {
                self.progress_done = *done;
                self.progress_total = *total;
            }
            TuiTaskEvent::ConfigTested { done, total, .. } => {
                self.progress_done = *done;
                self.progress_total = *total;
            }
            TuiTaskEvent::Completed { kind, message, .. } => {
                if self.running == Some(*kind) {
                    self.running = None;
                    self.cancellation = None;
                }
                self.completed_count += 1;
                self.last_summary = Some(message.clone());
                self.last_error = None;
            }
            TuiTaskEvent::Failed { kind, error, .. } => {
                if self.running == Some(*kind) {
                    self.running = None;
                    self.cancellation = None;
                }
                self.last_error = Some(error.clone());
            }
            TuiTaskEvent::Cancelled { kind } => {
                if self.running == Some(*kind) {
                    self.running = None;
                    self.cancellation = None;
                }
                self.last_summary = Some(format!("{:?} cancelled", kind));
            }
        }
    }
}
