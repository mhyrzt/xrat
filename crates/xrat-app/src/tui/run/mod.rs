mod input;
mod mouse;
mod tasks;
mod terminal;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use tasks::test_run_request_for_app;

use std::collections::BTreeSet;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::app::config::{ConfigEditSession, SettingEffect};
use crate::app::context::AppContext;
use crate::tui::app::{ConfirmKind, ConfirmState, SettingsModalState, TuiApp};
use crate::tui::data::TuiData;
use crate::tui::task::TuiTaskEvent;

use terminal::TerminalSession;

mod event_loop;
#[cfg(test)]
use event_loop::prepare_import_submission;
pub use event_loop::run;
