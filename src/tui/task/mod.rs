mod state;
#[cfg(test)]
mod tests;

use crate::tui::data::{TuiConfigRow, TuiData};

pub use state::TuiTaskState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiTaskKind {
    ReloadData,
    TestBatch,
    RuntimeOp,
    SourceRefresh,
}

#[derive(Debug)]
#[allow(dead_code, clippy::large_enum_variant)]
pub enum TuiTaskEvent {
    Started {
        kind: TuiTaskKind,
    },
    Progress {
        kind: TuiTaskKind,
        done: usize,
        total: usize,
    },
    ConfigTested {
        row: TuiConfigRow,
        done: usize,
        total: usize,
    },
    Completed {
        kind: TuiTaskKind,
        message: String,
        data: Option<TuiData>,
    },
    Failed {
        kind: TuiTaskKind,
        error: String,
        data: Option<TuiData>,
    },
    Cancelled {
        kind: TuiTaskKind,
    },
}
