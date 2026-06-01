use crate::tui::cancel::{TuiCancellationReceiver, TuiCancellationToken, new_cancellation};
use crate::tui::data::TuiData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiTaskKind {
    ReloadData,
    TestBatch,
}

#[derive(Debug)]
pub enum TuiTaskEvent {
    Started {
        kind: TuiTaskKind,
    },
    Completed {
        kind: TuiTaskKind,
        message: String,
        data: Option<TuiData>,
    },
    Failed {
        kind: TuiTaskKind,
        error: String,
    },
    Cancelled {
        kind: TuiTaskKind,
    },
}

#[derive(Debug, Default)]
pub struct TuiTaskState {
    pub running: Option<TuiTaskKind>,
    pub last_summary: Option<String>,
    pub last_error: Option<String>,
    pub completed_count: usize,
    pub cancellation: Option<TuiCancellationToken>,
}

impl TuiTaskState {
    pub fn label(&self) -> String {
        match self.running {
            Some(kind) if self.cancellation.as_ref().is_some_and(|t| t.is_cancelled()) => {
                format!("{:?} cancelling", kind)
            }
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
            TuiTaskEvent::Completed { kind, message, .. } => {
                if self.running == Some(*kind) {
                    self.running = None;
                    self.cancellation = None;
                }
                self.completed_count += 1;
                self.last_summary = Some(message.clone());
                self.last_error = None;
            }
            TuiTaskEvent::Failed { kind, error } => {
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

#[cfg(test)]
mod tests {
    use super::{TuiTaskEvent, TuiTaskKind, TuiTaskState};

    #[test]
    fn tracks_task_lifecycle() {
        let mut state = TuiTaskState::default();

        state.apply(&TuiTaskEvent::Started {
            kind: TuiTaskKind::ReloadData,
        });
        assert_eq!(state.running, Some(TuiTaskKind::ReloadData));
        assert_eq!(state.label(), "ReloadData running");

        state.apply(&TuiTaskEvent::Completed {
            kind: TuiTaskKind::ReloadData,
            message: "reloaded".to_string(),
            data: None,
        });

        assert_eq!(state.running, None);
        assert_eq!(state.completed_count, 1);
        assert_eq!(state.label(), "reloaded");
    }

    #[test]
    fn start_creates_cancellation_token() {
        let mut state = TuiTaskState::default();
        let (token, _receiver) = state.start(TuiTaskKind::TestBatch);

        assert!(!token.is_cancelled());
        assert_eq!(state.running, Some(TuiTaskKind::TestBatch));
    }

    #[test]
    fn cancel_only_succeeds_when_token_present() {
        let mut state = TuiTaskState::default();
        assert!(!state.cancel());

        let (_token, _receiver) = state.start(TuiTaskKind::TestBatch);
        assert!(state.cancel());
    }

    #[test]
    fn cancellation_clears_token() {
        let mut state = TuiTaskState::default();
        let _token = state.start(TuiTaskKind::TestBatch);

        state.apply(&TuiTaskEvent::Cancelled {
            kind: TuiTaskKind::TestBatch,
        });

        assert!(state.cancellation.is_none());
        assert_eq!(state.label(), "TestBatch cancelled");
    }
}
