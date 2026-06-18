use crossterm::event::KeyCode;

use crate::tui::app::{BulkOp, TestScope, TuiAction};

/// Chord leader keys. Config-management leaders are active in the Configs view;
/// `a` (API sharing) is active in all main TUI views. Pressing one arms a chord
/// that resolves on the next key.
pub fn leader_char(code: KeyCode) -> Option<char> {
    match code {
        KeyCode::Char('a') => Some('a'),
        KeyCode::Char('t') => Some('t'),
        KeyCode::Char('d') => Some('d'),
        KeyCode::Char('D') => Some('D'),
        KeyCode::Char('r') => Some('r'),
        KeyCode::Char('C') => Some('C'),
        _ => None,
    }
}

/// Resolve an armed chord `leader` against its second key. Unknown pairs (and
/// `Esc`) resolve to `None`, which simply clears the armed chord.
pub fn resolve_chord(leader: char, code: KeyCode) -> TuiAction {
    match (leader, code) {
        ('a', KeyCode::Char('q')) => TuiAction::OpenQrApiUrl,
        ('a', KeyCode::Char('c')) => TuiAction::CopyApiUrl,

        ('t', KeyCode::Char('t')) => TuiAction::StartTest(TestScope::Focused),
        ('t', KeyCode::Char('a')) => TuiAction::StartTest(TestScope::AllEnabled),
        ('t', KeyCode::Char('v')) => TuiAction::StartTest(TestScope::Filtered),
        ('t', KeyCode::Char('r')) => TuiAction::StartTest(TestScope::Failed),
        ('t', KeyCode::Char('s')) => TuiAction::StartTest(TestScope::Stale),
        ('t', KeyCode::Char('c')) => TuiAction::CancelTestBatch,

        ('d', KeyCode::Char('d')) => TuiAction::RequestDeleteFocused,
        ('d', KeyCode::Char('f')) => TuiAction::RequestBulk(BulkOp::DeleteFailed),
        ('d', KeyCode::Char('v')) => TuiAction::RequestBulk(BulkOp::DeleteFiltered),
        ('d', KeyCode::Char('x')) => TuiAction::RequestBulk(BulkOp::DeleteDisabled),

        ('D', KeyCode::Char('D')) => TuiAction::RequestPurgeFocused,
        ('D', KeyCode::Char('f')) => TuiAction::RequestBulk(BulkOp::PurgeFailed),
        ('D', KeyCode::Char('v')) => TuiAction::RequestBulk(BulkOp::PurgeFilteredDeleted),
        ('D', KeyCode::Char('a')) => TuiAction::RequestBulk(BulkOp::PurgeAllDeleted),

        ('r', KeyCode::Char('r')) => TuiAction::RestoreFocused,
        ('r', KeyCode::Char('v')) => TuiAction::RequestBulk(BulkOp::RestoreFilteredDeleted),
        ('r', KeyCode::Char('a')) => TuiAction::RequestBulk(BulkOp::RestoreAllDeleted),

        ('C', KeyCode::Char('l')) => TuiAction::ClearLogView,
        ('C', KeyCode::Char('s')) => TuiAction::ClearStatsView,
        ('C', KeyCode::Char('p')) => TuiAction::RequestClearEvents,

        _ => TuiAction::None,
    }
}

/// Human label for an armed chord leader, shown before its second-key hints.
pub fn chord_title(leader: char) -> &'static str {
    match leader {
        'a' => "API",
        't' => "TEST",
        'd' => "DELETE",
        'D' => "PURGE",
        'r' => "RESTORE",
        'C' => "CLEAR",
        _ => "",
    }
}

/// Second-key options for an armed chord: `(key, label)` pairs rendered in the
/// key bar so each key can be styled independently from its description.
pub fn chord_entries(leader: char) -> &'static [(&'static str, &'static str)] {
    match leader {
        'a' => &[("q", "QR"), ("c", "copy")],
        't' => &[
            ("t", "focused"),
            ("a", "all"),
            ("v", "filtered"),
            ("r", "failed"),
            ("s", "stale"),
            ("c", "cancel"),
        ],
        'd' => &[
            ("d", "focused"),
            ("f", "failed"),
            ("v", "filtered"),
            ("x", "disabled"),
        ],
        'D' => &[
            ("D", "focused"),
            ("f", "failed"),
            ("v", "filtered"),
            ("a", "trash"),
        ],
        'r' => &[("r", "focused"), ("v", "filtered"), ("a", "trash")],
        'C' => &[
            ("l", "log view"),
            ("s", "traffic view"),
            ("p", "events (db)"),
        ],
        _ => &[],
    }
}
