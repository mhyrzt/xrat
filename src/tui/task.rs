use crate::tui::data::TuiData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiTaskKind {
    ReloadData,
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
}

impl TuiTaskState {
    pub fn label(&self) -> String {
        match self.running {
            Some(kind) => format!("{:?} running", kind),
            None => self
                .last_summary
                .clone()
                .or_else(|| self.last_error.clone())
                .unwrap_or_else(|| "idle".to_string()),
        }
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
                }
                self.completed_count += 1;
                self.last_summary = Some(message.clone());
                self.last_error = None;
            }
            TuiTaskEvent::Failed { kind, error } => {
                if self.running == Some(*kind) {
                    self.running = None;
                }
                self.last_error = Some(error.clone());
            }
            TuiTaskEvent::Cancelled { kind } => {
                if self.running == Some(*kind) {
                    self.running = None;
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
}
