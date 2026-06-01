use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{TuiAction, TuiView};

pub fn action_for_key(
    key: KeyEvent,
    active_view: TuiView,
    editing_search: bool,
    confirming: bool,
) -> TuiAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return TuiAction::Quit;
    }

    if confirming {
        return match key.code {
            KeyCode::Enter | KeyCode::Char('y') => TuiAction::Confirm,
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => TuiAction::Cancel,
            _ => TuiAction::None,
        };
    }

    if editing_search {
        return match key.code {
            KeyCode::Esc => TuiAction::Back,
            KeyCode::Enter => TuiAction::ConfirmSearch,
            KeyCode::Backspace => TuiAction::SearchBackspace,
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                TuiAction::ClearSearch
            }
            KeyCode::Char(ch) => TuiAction::SearchInput(ch),
            _ => TuiAction::None,
        };
    }

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

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::action_for_key;
    use crate::tui::app::{TuiAction, TuiView};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn maps_global_quit_keys() {
        assert_eq!(
            action_for_key(key(KeyCode::Char('q')), TuiView::Configs, false, false),
            TuiAction::Quit
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                TuiView::Configs,
                false,
                false
            ),
            TuiAction::Quit
        );
    }

    #[test]
    fn maps_view_switching_keys() {
        assert_eq!(
            action_for_key(key(KeyCode::Char('1')), TuiView::Configs, false, false),
            TuiAction::SwitchView(TuiView::Configs)
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('4')), TuiView::Configs, false, false),
            TuiAction::SwitchView(TuiView::Runtime)
        );
    }

    #[test]
    fn maps_navigation_and_help_keys() {
        assert_eq!(
            action_for_key(key(KeyCode::Down), TuiView::Configs, false, false),
            TuiAction::MoveDown
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('k')), TuiView::Configs, false, false),
            TuiAction::MoveUp
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('?')), TuiView::Configs, false, false),
            TuiAction::ShowHelp
        );
        assert_eq!(
            action_for_key(key(KeyCode::Esc), TuiView::Configs, false, false),
            TuiAction::Back
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('/')), TuiView::Configs, false, false),
            TuiAction::BeginSearch
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('s')), TuiView::Configs, false, false),
            TuiAction::CycleSort
        );
    }

    #[test]
    fn maps_config_action_keys() {
        assert_eq!(
            action_for_key(key(KeyCode::Char(' ')), TuiView::Configs, false, false),
            TuiAction::SelectFocused
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('e')), TuiView::Configs, false, false),
            TuiAction::EnableFocused
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('x')), TuiView::Configs, false, false),
            TuiAction::DisableFocused
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('d')), TuiView::Configs, false, false),
            TuiAction::RequestDeleteFocused
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('D')), TuiView::Configs, false, false),
            TuiAction::RequestPurgeFocused
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('r')), TuiView::Configs, false, false),
            TuiAction::RestoreFocused
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('f')), TuiView::Configs, false, false),
            TuiAction::ToggleDeletedFilter
        );
    }

    #[test]
    fn maps_confirm_keys() {
        assert_eq!(
            action_for_key(key(KeyCode::Enter), TuiView::Configs, false, true),
            TuiAction::Confirm
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('y')), TuiView::Configs, false, true),
            TuiAction::Confirm
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('n')), TuiView::Configs, false, true),
            TuiAction::Cancel
        );
    }

    #[test]
    fn maps_search_editing_keys() {
        assert_eq!(
            action_for_key(key(KeyCode::Char('v')), TuiView::Configs, true, false),
            TuiAction::SearchInput('v')
        );
        assert_eq!(
            action_for_key(key(KeyCode::Backspace), TuiView::Configs, true, false),
            TuiAction::SearchBackspace
        );
        assert_eq!(
            action_for_key(key(KeyCode::Enter), TuiView::Configs, true, false),
            TuiAction::ConfirmSearch
        );
        assert_eq!(
            action_for_key(
                KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
                TuiView::Configs,
                true,
                false
            ),
            TuiAction::ClearSearch
        );
    }

    #[test]
    fn maps_tests_view_actions() {
        assert_eq!(
            action_for_key(key(KeyCode::Char('s')), TuiView::Tests, false, false),
            TuiAction::StartTestBatch
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('c')), TuiView::Tests, false, false),
            TuiAction::CancelTestBatch
        );
    }
}
