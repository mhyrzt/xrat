use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{TuiAction, TuiView};

pub fn action_for_key(key: KeyEvent) -> TuiAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return TuiAction::Quit;
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
        KeyCode::Char('/') => TuiAction::Search,
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
        assert_eq!(action_for_key(key(KeyCode::Char('q'))), TuiAction::Quit);
        assert_eq!(
            action_for_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            TuiAction::Quit
        );
    }

    #[test]
    fn maps_view_switching_keys() {
        assert_eq!(
            action_for_key(key(KeyCode::Char('1'))),
            TuiAction::SwitchView(TuiView::Configs)
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char('4'))),
            TuiAction::SwitchView(TuiView::Runtime)
        );
    }

    #[test]
    fn maps_navigation_and_help_keys() {
        assert_eq!(action_for_key(key(KeyCode::Down)), TuiAction::MoveDown);
        assert_eq!(action_for_key(key(KeyCode::Char('k'))), TuiAction::MoveUp);
        assert_eq!(action_for_key(key(KeyCode::Char('?'))), TuiAction::ShowHelp);
        assert_eq!(action_for_key(key(KeyCode::Esc)), TuiAction::Back);
        assert_eq!(action_for_key(key(KeyCode::Char('/'))), TuiAction::Search);
    }
}
