use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{TuiAction, TuiView};

pub fn action_for_view_key(key: KeyEvent, active_view: TuiView) -> TuiAction {
    match key.code {
        KeyCode::Char('q') => TuiAction::Quit,
        KeyCode::Esc => TuiAction::Back,
        KeyCode::Char('?') => TuiAction::ShowHelp,
        KeyCode::Char('1') => TuiAction::SwitchView(TuiView::Configs),
        KeyCode::Char('2') => TuiAction::SwitchView(TuiView::Sources),
        KeyCode::Char('3') => TuiAction::SwitchView(TuiView::Tests),
        KeyCode::Char('4') => TuiAction::SwitchView(TuiView::Runtime),
        KeyCode::Char('j') | KeyCode::Down => TuiAction::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => TuiAction::MoveUp,
        KeyCode::Char('/') => TuiAction::BeginSearch,
        KeyCode::Char('f') => TuiAction::ToggleDeletedFilter,
        KeyCode::Char('s') if active_view == TuiView::Tests => TuiAction::StartTestBatch,
        KeyCode::Char('c') if active_view == TuiView::Tests => TuiAction::CancelTestBatch,
        KeyCode::Char('s') => TuiAction::CycleSort,
        KeyCode::Char(' ') => TuiAction::SelectFocused,
        KeyCode::Char('e') => TuiAction::EnableFocused,
        KeyCode::Char('x') => TuiAction::DisableFocused,
        KeyCode::Char('d') => TuiAction::RequestDeleteFocused,
        KeyCode::Char('r') => TuiAction::RestoreFocused,
        KeyCode::Char('D') => TuiAction::RequestPurgeFocused,
        _ => TuiAction::None,
    }
}
