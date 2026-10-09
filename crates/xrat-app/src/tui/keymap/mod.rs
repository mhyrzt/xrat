mod chord;
pub use chord::{chord_entries, chord_title};
mod confirm;
mod search;
#[cfg(test)]
mod tests;
mod view;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{SettingsMode, SplitListTab, SplitModalMode, TuiAction, TuiPanel, TuiView};

mod bindings;
#[cfg(test)]
pub use bindings::action_for_key;
pub use bindings::{action_for_key_with_import, action_for_split_modal_key};
