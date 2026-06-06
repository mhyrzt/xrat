use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{TuiAction, TuiPanel, TuiView};

pub fn action_for_view_key(
    key: KeyEvent,
    active_view: TuiView,
    focused_panel: TuiPanel,
) -> TuiAction {
    match key.code {
        KeyCode::Char('q') => TuiAction::Quit,
        KeyCode::Esc => TuiAction::Back,
        KeyCode::Char('?') => TuiAction::ShowHelp,
        KeyCode::Left if active_view == TuiView::Configs && focused_panel == TuiPanel::Log => {
            TuiAction::PrevLogTab
        }
        KeyCode::Right if active_view == TuiView::Configs && focused_panel == TuiPanel::Log => {
            TuiAction::NextLogTab
        }
        KeyCode::Char('[') => TuiAction::PrevTab,
        KeyCode::Char(']') => TuiAction::NextTab,
        KeyCode::Tab => TuiAction::FocusNextPanel,
        KeyCode::BackTab => TuiAction::FocusPrevPanel,
        KeyCode::Char('j') | KeyCode::Down => TuiAction::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => TuiAction::MoveUp,
        KeyCode::Char('/') => TuiAction::BeginSearch,
        KeyCode::Char('T') => TuiAction::ToggleDeletedFilter,
        KeyCode::Char('F') => TuiAction::CycleFilter,
        KeyCode::Char('P') => TuiAction::CycleProtocolFilter,
        KeyCode::Enter if active_view == TuiView::Configs => TuiAction::StartFocused,
        KeyCode::Char('K') if active_view == TuiView::Configs => TuiAction::RuntimeStop,
        KeyCode::Char('R') if active_view == TuiView::Configs => TuiAction::RuntimeRestart,
        KeyCode::Char('r') if active_view == TuiView::Sources => TuiAction::RefreshFocusedSource,
        KeyCode::Char('R') if active_view == TuiView::Sources => TuiAction::RefreshAllSources,
        KeyCode::Char('n') if active_view == TuiView::Sources => TuiAction::OpenRenameModal,
        KeyCode::Char('d') if active_view == TuiView::Sources => TuiAction::RequestDeleteSource,
        KeyCode::Char('y') if active_view == TuiView::Sources => TuiAction::OpenQrFocused,
        KeyCode::Char('c') if active_view == TuiView::Sources => TuiAction::CopyFocused,
        KeyCode::Char('u') if active_view == TuiView::Sources => TuiAction::OpenQrApiUrl,
        KeyCode::Char('U') if active_view == TuiView::Sources => TuiAction::CopyApiUrl,
        KeyCode::Char('S') => TuiAction::CycleSort,
        KeyCode::Char('e') => TuiAction::EnableFocused,
        KeyCode::Char('x') => TuiAction::DisableFocused,
        KeyCode::Char('d') => TuiAction::RequestDeleteFocused,
        KeyCode::Char('r') => TuiAction::RestoreFocused,
        KeyCode::Char('D') => TuiAction::RequestPurgeFocused,
        KeyCode::Char('y') if active_view == TuiView::Configs => TuiAction::OpenQrFocused,
        KeyCode::Char('c') if active_view == TuiView::Configs => TuiAction::CopyFocused,
        _ => TuiAction::None,
    }
}
