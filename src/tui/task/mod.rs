mod state;
mod tests;

use crate::tui::data::TuiData;

pub use state::TuiTaskState;

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
